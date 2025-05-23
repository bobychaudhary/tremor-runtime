// Copyright 2020-2021, The Tremor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Identifies a node in the AST.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NodeId {
    /// The ID of the Node
    id: String,
    /// The module of the Node
    module: Vec<String>,
}

impl NodeId {
    /// Create a new `NodeId` from an ID and Module list.
    pub fn new(id: String, module: Vec<String>) -> Self {
        Self { id, module }
    }

    /// The node's id.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// The node's module.
    pub fn module(&self) -> &[String] {
        &self.module
    }

    /// Mutate the node's module.
    pub fn module_mut(&mut self) -> &mut Vec<String> {
        &mut self.module
    }

    /// Calculate the fully qualified name from
    /// the given module path.
    #[must_use]
    pub fn fqn(&self) -> String {
        if self.module.is_empty() {
            self.id.to_string()
        } else {
            format!("{}::{}", self.module.join("::"), self.id)
        }
    }

    /// Calculate the fully qualified name of some
    /// target identifier given this node's module
    /// path.
    #[must_use]
    pub fn target_fqn(&self, target: &str) -> String {
        if self.module.is_empty() {
            target.to_string()
        } else {
            format!("{}::{}", self.module.join("::"), target)
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_fqn {
    ($struct:ident) => {
        impl $struct<'_> {
            fn fqn(&self) -> String {
                self.node_id.fqn()
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::NodeId;

    #[test]
    fn fqn() {
        let no_module = NodeId::new("foo".to_string(), vec![]);
        assert_eq!(no_module.fqn(), "foo");
        assert!(no_module.module().is_empty());

        let with_module = NodeId::new(
            "foo".to_string(),
            vec!["bar".to_string(), "baz".to_string()],
        );
        assert_eq!(with_module.fqn(), "bar::baz::foo");
        assert_eq!(with_module.module(), &["bar", "baz"]);

        let target = "quux";
        assert_eq!(no_module.target_fqn(target), target);
        assert_eq!(with_module.target_fqn(target), "bar::baz::quux");
    }
}
