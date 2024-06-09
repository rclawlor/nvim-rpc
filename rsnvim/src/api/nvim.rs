use rmpv::Value;

use crate::api::Nvim;
use crate::error::Error;
use crate::value_vec;

impl Nvim {
    pub fn command(&mut self, command: String) -> Result<(), Error> {
        self.session
            .call("nvim_command", value_vec!(command))
            .unwrap();

        Ok(())
    }

    /// Since: 1
    pub fn subscribe(&mut self, event: String) -> Result<(), Error> {
        self.session.call(
            "nvim_subscribe",
            value_vec!(event)
        )?;

        Ok(())
    }
}
