use std::rc::Rc;

// TODO: give some gameplay effect to cards
// NOTE: fn (function pointer) is Copy, but closures might not be, both could work if passed the game state
// or should we just make this an enum and match the type in some logic state?
// NOTE: we do want to own all values, to enable returning this anywhere. We also want it to be Copy, or at least Clone.

// There's no pointer that owns its value and is Copy: to free a value, it needs to know when it's a last instance, and
// so it must count references; but refcounting can't be Copy, because the count would get out-of-sync.
#[derive(Clone)]
pub struct Card {
    name: Rc<String>,
}

impl Card {
    pub fn new(name: String) -> Self {
        Self {
            name: Rc::new(name)
        }
    }
}