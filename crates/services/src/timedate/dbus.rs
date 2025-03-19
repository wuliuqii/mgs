use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.timedate1",
    default_service = "org.freedesktop.timedate1",
    default_path = "/org/freedesktop/timedate1"
)]
pub trait TimeDate {
    /// A list of time zones known on the local system as an array of names.
    fn list_timezones(&self) -> zbus::Result<Vec<String>>;

    /// Control whether the RTC is in local time zone or UTC.
    #[zbus(name = "SetLocalRTC")]
    fn set_local_rtc(
        &self,
        local_rtc: bool,
        fix_system: bool,
        interactive: bool,
    ) -> zbus::Result<()>;

    /// Control whether the  pub(crate) system clock is synchronized with the network using `systemd-timesyncd`.
    #[zbus(name = "SetNTP")]
    fn set_ntp(&self, use_ntp: bool, interactive: bool) -> zbus::Result<()>;

    /// Change the system clock.
    fn set_time(&self, usec_utc: i64, relative: bool, interactive: bool) -> zbus::Result<()>;

    /// Set the system time zone.
    fn set_timezone(&self, timezone: &str, interactive: bool) -> zbus::Result<()>;

    /// Shows whether a service to perform time synchronization over network is available.
    #[zbus(property, name = "CanNTP")]
    fn can_ntp(&self) -> zbus::Result<bool>;

    /// Shows whether the RTC is configured to use UTC or the local time zone.
    #[zbus(property, name = "LocalRTC")]
    fn local_rtc(&self) -> zbus::Result<bool>;

    /// Shows whether the NTP service is enabled.
    #[zbus(property, name = "NTP")]
    fn ntp(&self) -> zbus::Result<bool>;

    /// Shows whether the kernel reports the time as synchronized.
    #[zbus(property, name = "NTPSynchronized")]
    fn ntp_synchronized(&self) -> zbus::Result<bool>;

    /// Shows the current time in RTC.
    #[zbus(property, name = "RTCTimeUSec")]
    fn rtctime_usec(&self) -> zbus::Result<u64>;

    /// Shows the current time.
    #[zbus(property, name = "TimeUSec")]
    fn time_usec(&self) -> zbus::Result<u64>;

    /// Shows the currently-configured time zone.
    #[zbus(property)]
    fn timezone(&self) -> zbus::Result<String>;
}
