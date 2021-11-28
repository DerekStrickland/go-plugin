pub mod plugin {
    include!("plugin.rs");
}
pub mod proto {
    include!("proto.rs");
}
pub mod google {
    pub mod protobuf {
        include!("google.protobuf.rs");
    }
}
