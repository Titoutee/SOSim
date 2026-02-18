/// All the context around an event loop, retrieving commands from the main CLI (or threaded CLI with -) on the fly,
/// in the manner of any CLI interpreter.
/// This permits to perform step by step allocation tests on mem, which makes it more handy to play with attacks following
/// a progressive scheme.

/// This module is CURRENTLY NOT IN USE

#[allow(unused)]
type Op = fn(String) -> ();

pub enum EventMode {
    Line { sep: char },
}

#[allow(unused)]
pub struct EventLoop {
    mode: EventMode,
    threaded: bool,
}

pub enum EventLoopTerm {
    None,
}

// Useless?
#[allow(unused)]
impl EventLoop {
    fn init_interpreter_loop(threaded: bool) -> Self {
        EventLoop {
            mode: EventMode::Line { sep: ';' },
            threaded,
        }
    }
    fn run(&self, f: Op) -> EventLoopTerm {
        EventLoopTerm::None
    }
}
