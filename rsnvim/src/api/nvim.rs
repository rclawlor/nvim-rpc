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
}
