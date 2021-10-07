pub mod base {
    tonic::include_proto!("hyperion.v1alpha1.base");
    pub type Any = prost_types::Any;
}

pub mod api {
    tonic::include_proto!("hyperion.v1alpha1.api");
}