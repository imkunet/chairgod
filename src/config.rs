use env_struct::env_struct;

env_struct! {
    #[derive(Clone)]
    pub struct ChairConfig {
        pub bot_token,
    }
}
