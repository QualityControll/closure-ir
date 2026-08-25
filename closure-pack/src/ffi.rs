use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalFunction {
    pub name: String,
    pub arguments: Vec<crate::types::TypeInfo>,
    pub return_type: crate::types::TypeInfo,
}

impl ExternalFunction {
    pub fn new<N: Into<String>>(
        name: N,
        arguments: Vec<crate::types::TypeInfo>,
        return_type: crate::types::TypeInfo,
    ) -> Self {
        Self {
            name: name.into(),
            arguments,
            return_type,
        }
    }
}
