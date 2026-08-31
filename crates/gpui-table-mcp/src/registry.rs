use super::{McpServer, McpTableDescriptor, McpToolError};

pub use inventory;

inventory::collect!(McpTableRegistration);
inventory::collect!(McpQueryHandlerRegistration);

pub struct McpTableRegistration {
    descriptor: fn() -> McpTableDescriptor,
}

impl McpTableRegistration {
    pub const fn new(descriptor: fn() -> McpTableDescriptor) -> Self {
        Self { descriptor }
    }

    pub fn descriptor(&self) -> McpTableDescriptor {
        (self.descriptor)()
    }
}

pub fn table_registrations() -> impl Iterator<Item = &'static McpTableRegistration> {
    inventory::iter::<McpTableRegistration>.into_iter()
}

pub struct McpQueryHandlerRegistration {
    register: fn(&mut McpServer) -> Result<(), McpToolError>,
}

impl McpQueryHandlerRegistration {
    pub const fn new(register: fn(&mut McpServer) -> Result<(), McpToolError>) -> Self {
        Self { register }
    }

    pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
        (self.register)(server)
    }
}

pub fn query_handler_registrations() -> impl Iterator<Item = &'static McpQueryHandlerRegistration> {
    inventory::iter::<McpQueryHandlerRegistration>.into_iter()
}
