use std::ops::Deref;

use super::dbus::active_connection::ActiveConnectionProxy;

#[derive(Debug)]
pub struct ActiveConnection<'a>(ActiveConnectionProxy<'a>);

impl<'a> Deref for ActiveConnection<'a> {
    type Target = ActiveConnectionProxy<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<ActiveConnectionProxy<'a>> for ActiveConnection<'a> {
    fn from(connection: ActiveConnectionProxy<'a>) -> Self {
        ActiveConnection(connection)
    }
}
