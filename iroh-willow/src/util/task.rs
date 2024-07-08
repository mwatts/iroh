//! Utilities for working with tokio tasks.

use std::{
    cell::RefCell,
    collections::HashMap,
    future::{poll_fn, Future},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_concurrency::future::{future_group, FutureGroup};
use futures_lite::{Stream, StreamExt};
use tokio::task::AbortHandle;
use tokio::task::JoinError;
use tracing::{Instrument, Span};

#[derive(derive_more::Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[debug("{:?}", _0)]
pub struct TaskKey(future_group::Key);

/// A collection of tasks spawned on a Tokio runtime, associated with hash map keys.
///
/// Similar to [`tokio::task::JoinSet`] but can also contain local tasks, and each task is
/// identified by a key which is returned upon completion of the task.
///
/// Uses [`tokio::task::spawn`] and [`tokio::task::spawn_local`] in combination with [`future_group`] for keeping the join handles around.
//
// TODO: Replace with [`tokio::task::JoinMap`] once it doesn't need tokio unstable anymore.
#[derive(Debug)]
pub struct JoinMap<K, T> {
    tasks: future_group::Keyed<tokio::task::JoinHandle<T>>,
    abort_handles: HashMap<TaskKey, AbortHandle>,
    keys: HashMap<TaskKey, K>,
}

impl<K, T> Default for JoinMap<K, T> {
    fn default() -> Self {
        Self {
            tasks: FutureGroup::new().keyed(),
            keys: Default::default(),
            abort_handles: Default::default(),
        }
    }
}

impl<K: Unpin, T: 'static> JoinMap<K, T> {
    /// Create a new [`TaskMap`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Spawn a new task on the currently executing [`tokio::task::LocalSet`].
    pub fn spawn_local<F: Future<Output = T> + 'static>(&mut self, key: K, future: F) -> TaskKey {
        let handle = tokio::task::spawn_local(future);
        let abort_handle = handle.abort_handle();
        let k = TaskKey(self.tasks.insert(handle));
        self.keys.insert(k, key);
        self.abort_handles.insert(k, abort_handle);
        k
    }

    /// Poll for one of the tasks in the map to complete.
    pub fn poll_join_next(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<(K, Result<T, JoinError>)>> {
        let Some((key, item)) = std::task::ready!(Pin::new(&mut self.tasks).poll_next(cx)) else {
            return Poll::Ready(None);
        };
        let key = self.keys.remove(&TaskKey(key)).expect("key to exist");
        Poll::Ready(Some((key, item)))
    }

    /// Remove a task from the map.
    pub fn remove(&mut self, task_key: &TaskKey) -> bool {
        self.keys.remove(task_key);
        self.tasks.remove(task_key.0)
    }

    /// Returns `true` if the task map is currently empty.
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Returns the number of tasks currently in the map.
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &TaskKey)> {
        self.keys.iter().map(|(a, b)| (b, a))
    }

    pub fn abort_all(&mut self) {
        for (_, handle) in self.abort_handles.drain() {
            handle.abort();
        }
    }

    pub async fn shutdown(&mut self) {
        self.abort_all();
        while self.next().await.is_some() {}
    }
}

impl<K, T: Send + 'static> JoinMap<K, T> {
    /// Spawn a new, non-local task on the current tokio runtime.
    pub fn spawn<F: Future<Output = T> + 'static + Send>(&mut self, future: F) -> TaskKey {
        let handle = tokio::task::spawn(future);
        let key = self.tasks.insert(handle);
        TaskKey(key)
    }
}

impl<K: Unpin, T: 'static> Stream for JoinMap<K, T> {
    type Item = (K, Result<T, JoinError>);

    /// Poll for one of the tasks to complete.
    ///
    /// See [`Self::poll_join_next`] for details.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Self::poll_join_next(self.get_mut(), cx)
    }
}

#[derive(Debug)]
pub struct SharedJoinMap<K, T>(Rc<RefCell<JoinMap<K, T>>>);

impl<K, T> Clone for SharedJoinMap<K, T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<K, T> Default for SharedJoinMap<K, T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, T> SharedJoinMap<K, T>
where
    K: Unpin,
    T: 'static,
{
    pub async fn join_next(&self) -> Option<(K, Result<T, JoinError>)> {
        poll_fn(|cx| {
            let mut tasks = self.0.borrow_mut();
            let res = std::task::ready!(Pin::new(&mut tasks).poll_join_next(cx));
            Poll::Ready(res)
        })
        .await
    }

    pub fn abort_all(&self) {
        self.0.borrow_mut().abort_all();
    }

    pub async fn shutdown(&self) {
        self.abort_all();
        while let Some(_) = self.join_next().await {}
    }
}

impl<T: 'static> SharedJoinMap<Span, T> {
    pub fn spawn<Fut>(&self, span: Span, fut: Fut)
    where
        Fut: std::future::Future<Output = T> + 'static,
    {
        let fut = fut.instrument(span.clone());
        self.0.borrow_mut().spawn_local(span, fut);
    }

    pub fn remaining_tasks(&self) -> String {
        let tasks = self.0.borrow();
        let mut out = vec![];
        for (span, _k) in tasks.iter() {
            let name = span.metadata().unwrap().name();
            out.push(name.to_string());
        }
        out.join(",")
    }

    pub fn log_remaining_tasks(&self) {
        let tasks = self.0.borrow();
        let names = tasks
            .iter()
            .map(|t| t.0.metadata().unwrap().name())
            .collect::<Vec<_>>();
        tracing::debug!(tasks=?names, "active_tasks");
    }
}