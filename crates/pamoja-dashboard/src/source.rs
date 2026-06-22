//! The single seam between the dashboard and whatever produces its data.
//!
//! A real node and the [`Mock`](crate::Mock) both implement [`StateSource`], so the
//! serving layer never knows which it is talking to. What you design and debug against
//! the mock on a laptop is exactly what ships against real sensors.

use crate::state::State;

/// Produces the current [`State`] snapshot whenever the dashboard asks for one.
///
/// The serving layer calls [`snapshot`](StateSource::snapshot) to answer `GET /state`
/// and again on each live-update tick, so an implementation should return the latest
/// view of the node cheaply. It takes `&mut self` so a source may advance internal
/// state (a mock its clock, a real node its smoothing) as it is polled.
pub trait StateSource {
    /// Returns the node's current state snapshot.
    ///
    /// # Returns
    ///
    /// The latest language-neutral [`State`] to render.
    fn snapshot(&mut self) -> State;

    /// Switches a named view, for development and debugging only.
    ///
    /// The serving layer calls this when a request carries a `?scenario=` parameter,
    /// so a single running dev server can be flipped through every state the UI must
    /// handle. A real node has nothing to switch, so the default ignores the request.
    ///
    /// # Arguments
    ///
    /// * `key` - the requested view's identifier.
    ///
    /// # Returns
    ///
    /// `true` if the source switched to `key`, `false` if it does not recognize it.
    fn select(&mut self, key: &str) -> bool {
        let _ = key;
        false
    }
}
