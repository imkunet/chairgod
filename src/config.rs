use env_struct::env_struct;

env_struct! {
    #[derive(Clone)]
    pub(crate) struct ChairConfig {
        pub(crate) bot_token,
    }
}
