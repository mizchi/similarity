use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub value: String,
    pub children: Vec<Rc<TreeNode>>,
    pub id: usize,
    pub subtree_size: Option<usize>,
}

impl TreeNode {
    #[must_use]
    pub fn new(label: String, value: String, id: usize) -> Self {
        TreeNode { label, value, children: Vec::new(), id, subtree_size: None }
    }

    pub fn add_child(&mut self, child: Rc<TreeNode>) {
        self.children.push(child);
    }

    #[must_use]
    pub fn get_subtree_size(&self) -> usize {
        if let Some(size) = self.subtree_size {
            return size;
        }
        let mut size = 1;
        for child in &self.children {
            size += child.get_subtree_size();
        }
        size
    }
}
