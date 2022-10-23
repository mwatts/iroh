include_proto!("one");

proxy!(
    One,
    version: () => VersionResponse => VersionResponse
);
