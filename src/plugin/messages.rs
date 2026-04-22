use pastey::paste;

macro_rules! xplm_message {
    [$($name:ident),*] => {
        $(
            paste! {
                pub const [<$name:snake:upper>]: i32 = xplm_sys::[<XPLM_MSG_$name:snake:upper>].cast_signed();
            }
        )+

        paste! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[repr(i32)]
            pub enum Message {
                $(
                        $name = [<$name:snake:upper>]
                ),+,
                Custom(i32)
            }

            impl<T> From<T> for Message
            where
                T: Into<i32>,
            {
                fn from(value: T) -> Self {
                    match value.into() {
                        $(
                            [<$name:snake:upper>] => Self::$name,
                        )+
                        other => Self::Custom(other),
                    }
                }
            }
        }
    };
}

xplm_message![
    PlaneCrashed,
    PlaneLoaded,
    AirportLoaded,
    SceneryLoaded,
    AirplaneCountChanged,
    PlaneUnloaded,
    WillWritePrefs,
    LiveryLoaded,
    EnteredVr,
    ExitingVr,
    ReleasePlanes,
    FmodBankLoaded,
    FmodBankUnloading,
    DatarefsAdded
];
