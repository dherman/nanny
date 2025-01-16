mod console;
mod process;
use crate::context::{Context, Cx, FunctionContext, ModuleContext};

pub use console::Console;
pub use process::Process;

pub trait Node<'cx>: Context<'cx> {
    fn console<'a>(&'a mut self) -> Console<'a, 'cx, Self> {
        Console::new(self)
    }

    fn process<'a>(&'a mut self) -> Process<'a, 'cx, Self> {
        Process::new(self)
    }
}

impl<'cx> Node<'cx> for ModuleContext<'cx> {}

impl<'cx> Node<'cx> for FunctionContext<'cx> {}

impl<'cx> Node<'cx> for Cx<'cx> {}
