use std::collections::{BinaryHeap, VecDeque};

use image::{Rgb, RgbImage, Rgba, RgbaImage};

use crate::image::ImageData;

struct NodeChildren {
    nw: usize,
    ne: usize,
    sw: usize,
    se: usize,
}

// children are stored as indexes in node array
struct Node {
    top_left: (usize, usize),
    bottom_right: (usize, usize),

    children: Option<NodeChildren>,
}

impl Node {
    pub fn leaf(top_left: (usize, usize), bottom_right: (usize, usize)) -> Self {
        Self {
            top_left,
            bottom_right,
            children: None,
        }
    }

    fn height(&self) -> u64 {
        (self.bottom_right.0 as u64) - (self.top_left.0 as u64)
    }

    fn width(&self) -> u64 {
        (self.bottom_right.1 as u64) - (self.top_left.1 as u64)
    }

    fn can_split(&self) -> bool {
        self.width() > 1 && self.height() > 1
    }

    fn split(&self) -> Option<(Node, Node, Node, Node)> {
        // guarantees that node is split into 4 children
        if !self.can_split() {
            return None;
        }

        let split_h = (self.top_left.0 + self.bottom_right.0) / 2;
        let split_w = (self.top_left.1 + self.bottom_right.1) / 2;

        let nw_node = Node::leaf(self.top_left, (split_h, split_w));
        let ne_node = Node::leaf(
            (self.top_left.0, split_w + 1),
            (split_h, self.bottom_right.1),
        );
        let sw_node = Node::leaf(
            (split_h + 1, self.top_left.1),
            (self.bottom_right.0, split_w),
        );
        let se_node = Node::leaf((split_h + 1, split_w + 1), self.bottom_right);

        Some((nw_node, ne_node, sw_node, se_node))
    }
}

struct OrdNode {
    node_index: usize,
    metric: u64,
}

impl OrdNode {
    pub fn new(nodes: &Vec<Node>, index: usize, image_data: &ImageData) -> Self {
        let top_left = nodes[index].top_left;
        let bottom_right = nodes[index].bottom_right;
        Self {
            node_index: index,
            metric: image_data.metric(top_left, bottom_right),
        }
    }
}

impl PartialEq for OrdNode {
    fn eq(&self, other: &Self) -> bool {
        self.metric == other.metric
    }
}

impl Eq for OrdNode {}

impl PartialOrd for OrdNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.metric.cmp(&other.metric))
    }
}

impl Ord for OrdNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.metric.cmp(&other.metric)
    }
}

pub struct Tree {
    image_data: ImageData,
    nodes: Vec<Node>,
    pq: BinaryHeap<OrdNode>,
    dimensions: (usize, usize),
}

const MAX_ALPHA: u8 = 100;

impl Tree {
    pub fn new(image_data: ImageData) -> Self {
        let dimensions = (image_data.height(), image_data.width());
        let root = Node::leaf((0, 0), (dimensions.0 - 1, dimensions.1 - 1));
        let nodes = vec![root];
        let mut pq = BinaryHeap::new();
        pq.push(OrdNode::new(&nodes, 0, &image_data));

        Self {
            image_data,
            nodes,
            pq,
            dimensions,
        }
    }

    fn push_node(&mut self, node: Node) -> usize {
        let ret = self.nodes.len();
        self.nodes.push(node);
        ret
    }

    pub fn refine(&mut self) -> Result<(), String> {
        loop {
            let Some(top) = self.pq.pop() else {
                return Err("no more nodes to refine".into());
            };

            if let Some((nw, ne, sw, se)) = self.nodes[top.node_index].split() {
                let nw_index = self.push_node(nw);
                let ne_index = self.push_node(ne);
                let sw_index = self.push_node(sw);
                let se_index = self.push_node(se);

                let top_node_mut = &mut self.nodes[top.node_index];
                top_node_mut.children = Some(NodeChildren {
                    nw: nw_index,
                    ne: ne_index,
                    sw: sw_index,
                    se: se_index,
                });

                for ind in [nw_index, ne_index, sw_index, se_index].into_iter() {
                    self.pq
                        .push(OrdNode::new(&self.nodes, ind, &self.image_data));
                }
                return Ok(());
            }
            // else can't split, go again
        }
    }

    pub fn render_rgb(&self) -> RgbImage {
        let (h, w) = self.dimensions;
        let mut ret = RgbImage::new(w as u32, h as u32);

        let mut q = VecDeque::new();
        q.push_back(0);
        while let Some(cur) = q.pop_front() {
            let node = &self.nodes[cur];
            if let Some(NodeChildren { nw, ne, sw, se }) = node.children {
                q.push_back(nw);
                q.push_back(ne);
                q.push_back(sw);
                q.push_back(se);
            } else {
                let (start_y, start_x) = node.top_left;
                let (end_y, end_x) = node.bottom_right;
                let color = self.image_data.average(node.top_left, node.bottom_right);
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        ret.put_pixel(
                            x as u32,
                            y as u32,
                            Rgb([color.r as u8, color.g as u8, color.b as u8]),
                        );
                    }
                }
            }
        }

        return ret;
    }

    pub fn render_rgba(&self) -> RgbaImage {
        let (h, w) = self.dimensions;
        let mut ret = RgbaImage::new(w as u32, h as u32);

        let mut q = VecDeque::new();
        q.push_back(0);
        while let Some(cur) = q.pop_front() {
            let node = &self.nodes[cur];
            if let Some(NodeChildren { nw, ne, sw, se }) = node.children {
                q.push_back(nw);
                q.push_back(ne);
                q.push_back(sw);
                q.push_back(se);
            } else {
                let (start_y, start_x) = node.top_left;
                let (end_y, end_x) = node.bottom_right;
                let color = self.image_data.average(node.top_left, node.bottom_right);
                for x in start_x..=end_x {
                    for y in start_y..=end_y {
                        ret.put_pixel(
                            x as u32,
                            y as u32,
                            Rgba([color.r as u8, color.g as u8, color.b as u8, MAX_ALPHA]),
                        );
                    }
                }
            }
        }

        return ret;
    }
}
