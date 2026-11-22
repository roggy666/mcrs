use alloc::vec::Vec;

use crate::node::Node;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Heap {
    order: Vec<u32>,
}

impl Heap {
    pub(crate) fn clear(&mut self) {
        self.order.clear();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    fn f(nodes: &[Node], index: u32) -> f32 {
        nodes
            .get(index as usize)
            .map_or(f32::INFINITY, |node| node.f)
    }

    fn place(nodes: &mut [Node], order: &[u32], at: usize) {
        if let Some(&index) = order.get(at)
            && let Some(node) = nodes.get_mut(index as usize)
        {
            node.heap = u32::try_from(at).ok();
        }
    }

    pub(crate) fn insert(&mut self, nodes: &mut [Node], index: u32) {
        self.order.push(index);
        let at = self.order.len() - 1;
        Self::place(nodes, &self.order, at);
        self.up(nodes, at);
    }

    pub(crate) fn pop(&mut self, nodes: &mut [Node]) -> Option<u32> {
        let first = *self.order.first()?;
        let last = self.order.pop()?;
        if let Some(node) = nodes.get_mut(first as usize) {
            node.heap = None;
        }
        if !self.order.is_empty() {
            if let Some(slot) = self.order.first_mut() {
                *slot = last;
            }
            Self::place(nodes, &self.order, 0);
            self.down(nodes, 0);
        }
        Some(first)
    }

    pub(crate) fn change_key(&mut self, nodes: &mut [Node], index: u32, key: f32) {
        let Some(at) = nodes
            .get(index as usize)
            .and_then(|node| node.heap)
            .map(|at| at as usize)
        else {
            return;
        };
        let old = Self::f(nodes, index);
        if let Some(node) = nodes.get_mut(index as usize) {
            node.f = key;
        }
        if key < old {
            self.up(nodes, at);
        } else {
            self.down(nodes, at);
        }
    }

    fn up(&mut self, nodes: &mut [Node], mut at: usize) {
        while at > 0 {
            let parent = (at - 1) / 2;
            let (Some(&own), Some(&above)) = (self.order.get(at), self.order.get(parent)) else {
                break;
            };
            if Self::f(nodes, own) < Self::f(nodes, above) {
                self.order.swap(at, parent);
                Self::place(nodes, &self.order, at);
                Self::place(nodes, &self.order, parent);
                at = parent;
            } else {
                break;
            }
        }
    }

    fn down(&mut self, nodes: &mut [Node], mut at: usize) {
        let len = self.order.len();
        loop {
            let left = 2 * at + 1;
            let right = 2 * at + 2;
            if left >= len {
                break;
            }
            let left_f = self
                .order
                .get(left)
                .map_or(f32::INFINITY, |&i| Self::f(nodes, i));
            let right_f = self
                .order
                .get(right)
                .map_or(f32::INFINITY, |&i| Self::f(nodes, i));
            let (candidate, candidate_f) = if left_f < right_f {
                (left, left_f)
            } else {
                (right, right_f)
            };
            let own_f = self
                .order
                .get(at)
                .map_or(f32::INFINITY, |&i| Self::f(nodes, i));
            if candidate < len && candidate_f < own_f {
                self.order.swap(at, candidate);
                Self::place(nodes, &self.order, at);
                Self::place(nodes, &self.order, candidate);
                at = candidate;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use mcrs_voxel::BlockPos;

    fn node(f: f32) -> Node {
        Node {
            f,
            ..Node::new(BlockPos::new(0, 0, 0))
        }
    }

    #[test]
    fn the_heap_pops_the_smallest_f_in_the_order_of_the_reference_heap() {
        let mut nodes = vec![node(3.0), node(1.0), node(2.0), node(1.0), node(0.5)];
        let mut heap = Heap::default();
        for index in 0..nodes.len() as u32 {
            heap.insert(&mut nodes, index);
        }
        let mut popped = Vec::new();
        while let Some(index) = heap.pop(&mut nodes) {
            popped.push(index);
        }
        assert_eq!(popped, vec![4, 3, 1, 2, 0]);
    }

    #[test]
    fn a_key_change_moves_a_node_up_or_down() {
        let mut nodes = vec![node(1.0), node(2.0), node(3.0)];
        let mut heap = Heap::default();
        for index in 0..3 {
            heap.insert(&mut nodes, index);
        }
        heap.change_key(&mut nodes, 2, 0.5);
        assert_eq!(heap.pop(&mut nodes), Some(2));
        heap.change_key(&mut nodes, 0, 5.0);
        assert_eq!(heap.pop(&mut nodes), Some(1));
        assert_eq!(heap.pop(&mut nodes), Some(0));
        assert!(heap.is_empty());
    }
}
