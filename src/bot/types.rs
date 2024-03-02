use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

pub type State = ();
pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

pub type HandlerErr = Box<dyn std::error::Error + Send + Sync>;
pub type HandlerResult = Result<(), HandlerErr>;
