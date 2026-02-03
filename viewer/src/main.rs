use geometry_core::search::{SearchEngine, HeapFrontier,Frontier,SearchNode};
use geometry_core::layout::LayoutState;
use std::{thread, time::Duration};

#[derive(Clone)]
struct DummyNode(i32);

impl SearchNode for DummyNode {
    fn expand(&self) -> Vec<Self> {
        vec![DummyNode(self.0 + 1)]
    }

    fn score(&self) -> f32 {
        self.0 as f32
    }

    fn key(&self) -> u64 {
        self.0 as u64
    }
}

fn main() {
    let root = DummyNode(0);
    let frontier = HeapFrontier::new();

    let mut engine = SearchEngine::with_root(frontier, root);

    loop {
        step(&mut engine);
        render(&engine);

        thread::sleep(Duration::from_millis(300));
    }
}

fn step<T, F>(engine: &mut SearchEngine<T, F>)
where
    T: SearchNode,
    F: Frontier<T>,
{
    engine.step();
}

fn render<T, F>(engine: &SearchEngine<T, F>)
where
    T: SearchNode,
    F: Frontier<T>,
{
    println!("Frontier size = {}", engine.frontier_len());
}
