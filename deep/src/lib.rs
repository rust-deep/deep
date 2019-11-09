mod tensor;

pub use tensor::Tensor;

use ndarray::Axis;

#[derive(Clone, Debug)]
pub struct Internal {
    /// The node to pull the input tensor from.
    node: usize,
    /// The specific output to pull from.
    output: usize,
}

impl Internal {
    fn shift_inputs(&mut self, shift: usize) {
        self.node += shift;
    }
}

#[derive(Clone, Debug)]
pub enum Op {
    Add(Input, Input),
    Sub(Input, Input),
    MeanPool(Axis),
}

impl Op {
    fn shift_inputs(&mut self, shift: usize) {
        match self {
            Self::Add(a, b) => {
                a.shift_inputs(shift);
                b.shift_inputs(shift);
            }
            Self::Sub(a, b) => {
                a.shift_inputs(shift);
                b.shift_inputs(shift);
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    // An input from the feed dict.
    Feed(String),
    // An input from another node in the graph.
    Internal(Internal),
}

impl Input {
    fn shift_inputs(&mut self, shift: usize) {
        if let Self::Internal(n) = self {
            n.shift_inputs(shift);
        }
    }
}

impl From<&str> for Input {
    fn from(s: &str) -> Input {
        Input::Feed(s.to_owned())
    }
}

#[derive(Clone, Default, Debug)]
pub struct Graph {
    /// A series of ops refering to each other's outputs for their input.
    ops: Vec<Op>,
}

impl Graph {
    fn merge(&mut self, other: Graph) {
        let current = self.ops.len();
        self.ops.extend(other.ops);
        for op in &mut self.ops[current..] {
            op.shift_inputs(current);
        }
    }

    fn merge_input(&mut self, other: Graph, mut input: Input) -> Input {
        let current = self.ops.len();
        self.merge(other);
        input.shift_inputs(current);
        input
    }
}

trait Backend {
    type Inputs;
    type Internal;
    type Output;
    type Delta;

    /// Gets the output of solving the requested tensor.
    fn forward(
        &self,
        graph: &Graph,
        inputs: Self::Inputs,
        tensor: Input,
    ) -> (Self::Output, Self::Internal);

    /// Propogates a delta from the output back to the input via chain rule
    /// and produces a `Delta` that can be used to update the graph
    /// with an optimizer. The `Delta` contains all the dE/dx of all internal
    /// variables.
    fn backward(
        &self,
        graph: &Graph,
        internal: &Self::Internal,
        inputs: Self::Inputs,
        tensor: Input,
        output_delta: &Self::Output,
    ) -> Self::Delta;

    /// Applies a delta to the graph.
    fn train(&self, graph: &mut Graph, delta: &Self::Delta);
}