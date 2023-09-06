mod error;
pub use error::{PushRequestError, PushRequestResult};
mod fpush_config;
pub use fpush_config::FpushPushConfig;
pub use fpush_config::PushConfig;

mod push_handler;
pub use push_handler::handle_push_request;
mod push_module;

use dashmap::DashMap;
use push_module::{PushModule, PushModuleEnum, PushModuleMapArc};
use std::sync::Arc;

use log::{debug, info};

pub type FpushPushArc = Arc<FpushPush>;

pub struct FpushPush {
    push_modules: PushModuleMapArc,
}

impl FpushPush {
    pub async fn new(module_config: &FpushPushConfig) -> Self {
        let mut a = Self {
            push_modules: Arc::new(DashMap::default()),
        };
        a.load_push_modules(module_config).await;
        a
    }

    async fn load_push_modules(&mut self, module_config: &FpushPushConfig) {
        let mut default_counter = 0;
        for (push_module_id, module_config) in module_config.config() {
            let (is_default_module, push_module) =
                Self::init_push_module(push_module_id.clone(), module_config).await;
            self.push_modules
                .insert(push_module_id.to_string(), push_module);
            if is_default_module {
                default_counter += 1;
                info!("Loading {} as default push module", push_module_id);
                let (_, push_module) =
                    Self::init_push_module(push_module_id.clone(), module_config).await;
                self.push_modules.insert("default".to_string(), push_module);
            }
        }
        if default_counter > 1 {
            panic!("At most one push module can be defined as the default module");
        }
    }

    /// Load and init push module using the provided configuration
    /// Return true if the push module is the default push module
    async fn init_push_module(key: String, module_config: &PushConfig) -> (bool, PushModuleEnum) {
        match module_config {
            #[cfg(feature = "enable_apns_support")]
            PushConfig::Apple {
                apns,
                blacklist,
                ratelimit,
                is_default_module,
            } => {
                let apple_push_module =
                    PushModule::new_apple_module(key.clone(), apns, blacklist, ratelimit).unwrap();
                (*is_default_module, PushModuleEnum::Apple(apple_push_module))
            }
            #[cfg(feature = "enable_fcm_support")]
            PushConfig::Google {
                fcm,
                blacklist,
                ratelimit,
                is_default_module,
            } => {
                let google_fcm_push_module =
                    PushModule::new_fcm_module(key.clone(), fcm, blacklist, ratelimit)
                        .await
                        .unwrap();
                (
                    *is_default_module,
                    PushModuleEnum::Google(google_fcm_push_module),
                )
            }
            #[cfg(feature = "enable_demo_support")]
            PushConfig::Demo {
                blacklist,
                ratelimit,
                is_default_module,
            } => {
                let demo = PushModule::new_demo_module(key.clone(), blacklist, ratelimit)
                    .await
                    .unwrap();
                (*is_default_module, PushModuleEnum::Demo(demo))
            }
        }
    }

    #[inline(always)]
    pub async fn push(&self, module_id: &str, token: String) -> PushRequestResult<()> {
        if let Some(push_module) = self.push_modules.get(module_id) {
            handle_push_request(push_module.value(), token).await
        } else {
            debug!("Unknown push_module requested: {}", module_id);
            Err(PushRequestError::UnknownPushModule)
        }
    }
}
