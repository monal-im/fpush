use std::sync::Arc;

use crate::error::Result;
use fpush_ratelimit::{FpushTokenRateLimit, RatelimitSettings};
use fpush_tokenblocker::BlacklistSettings;
use fpush_tokenblocker::FpushBlocklist;

use fpush_traits::push::PushResult;

use dashmap::DashMap;
use fpush_traits::push::PushTrait;

pub type PushModuleMapArc = Arc<DashMap<String, PushModuleEnum>>;

pub enum PushModuleEnum {
    #[cfg(feature = "enable_apns_support")]
    Apple(PushModule<fpush_apns::FpushApns>),
    #[cfg(feature = "enable_fcm_support")]
    Google(PushModule<fpush_fcm::FpushFcm>),
    #[cfg(feature = "enable_demo_support")]
    Demo(PushModule<fpush_demopush::FpushDemoPush>),
}

impl PushModuleEnum {
    /// dispatch
    #[inline(always)]
    pub async fn send(&self, token: String) -> PushResult<()> {
        match self {
            #[cfg(feature = "enable_apns_support")]
            PushModuleEnum::Apple(push_module) => push_module.send(token).await,
            #[cfg(feature = "enable_fcm_support")]
            PushModuleEnum::Google(push_module) => push_module.send(token).await,
            #[cfg(feature = "enable_demo_support")]
            PushModuleEnum::Demo(push_module) => push_module.send(token).await,
        }
    }

    #[inline(always)]
    pub fn blocklist(&self) -> &Arc<FpushBlocklist> {
        match self {
            #[cfg(feature = "enable_apns_support")]
            PushModuleEnum::Apple(push_module) => push_module.blocklist(),
            #[cfg(feature = "enable_fcm_support")]
            PushModuleEnum::Google(push_module) => push_module.blocklist(),
            #[cfg(feature = "enable_demo_support")]
            PushModuleEnum::Demo(push_module) => push_module.blocklist(),
        }
    }

    #[inline(always)]
    pub fn ratelimit(&self) -> &Arc<FpushTokenRateLimit> {
        match self {
            #[cfg(feature = "enable_apns_support")]
            PushModuleEnum::Apple(push_module) => push_module.ratelimit(),
            #[cfg(feature = "enable_fcm_support")]
            PushModuleEnum::Google(push_module) => push_module.ratelimit(),
            #[cfg(feature = "enable_demo_support")]
            PushModuleEnum::Demo(push_module) => push_module.ratelimit(),
        }
    }

    #[inline(always)]
    pub fn identifier(&self) -> &str {
        match self {
            #[cfg(feature = "enable_apns_support")]
            PushModuleEnum::Apple(push_module) => push_module.identifier(),
            #[cfg(feature = "enable_fcm_support")]
            PushModuleEnum::Google(push_module) => push_module.identifier(),
            #[cfg(feature = "enable_demo_support")]
            PushModuleEnum::Demo(push_module) => push_module.identifier(),
        }
    }
}

pub struct PushModule<T>
where
    T: PushTrait,
{
    blocklist: Arc<FpushBlocklist>,
    token_ratelimit: Arc<FpushTokenRateLimit>,
    push: Arc<T>,
    identifier: String,
}

#[cfg(feature = "enable_apns_support")]
impl PushModule<fpush_apns::FpushApns> {
    pub(crate) fn new_apple_module(
        identifier: String,
        apns_conf: &fpush_apns::AppleApnsConfig,
        blocklist_config: &BlacklistSettings,
        ratelimit_config: &RatelimitSettings,
    ) -> Result<PushModule<fpush_apns::FpushApns>> {
        let apple_push = fpush_apns::FpushApns::init(apns_conf)?;
        Self::new(
            identifier,
            blocklist_config,
            ratelimit_config,
            Arc::new(apple_push),
        )
    }
}

#[cfg(feature = "enable_fcm_support")]
impl PushModule<fpush_fcm::FpushFcm> {
    pub(crate) async fn new_fcm_module(
        identifier: String,
        fcm_conf: &fpush_fcm::GoogleFcmConfig,
        blocklist_config: &BlacklistSettings,
        ratelimit_config: &RatelimitSettings,
    ) -> Result<PushModule<fpush_fcm::FpushFcm>> {
        let fcm_push = fpush_fcm::FpushFcm::init(fcm_conf).await?;
        Self::new(
            identifier,
            blocklist_config,
            ratelimit_config,
            Arc::new(fcm_push),
        )
    }
}

#[cfg(feature = "enable_demo_support")]
impl PushModule<fpush_demopush::FpushDemoPush> {
    pub(crate) async fn new_demo_module(
        identifier: String,
        blocklist_config: &BlacklistSettings,
        ratelimit_config: &RatelimitSettings,
    ) -> Result<PushModule<fpush_demopush::FpushDemoPush>> {
        let demo_module = fpush_demopush::FpushDemoPush::init()?;
        Self::new(
            identifier,
            blocklist_config,
            ratelimit_config,
            Arc::new(demo_module),
        )
    }
}

impl<T> PushModule<T>
where
    T: PushTrait,
{
    /// Create new push module of type <T>
    pub(crate) fn new(
        identifier: String,
        blocklist_config: &BlacklistSettings,
        ratelimit_config: &RatelimitSettings,
        push: Arc<T>,
    ) -> Result<Self> {
        let blocklist = fpush_tokenblocker::FpushBlocklist::new(blocklist_config);

        let token_ratelimit = fpush_ratelimit::FpushTokenRateLimit::new(ratelimit_config);

        let module = Self {
            blocklist: Arc::new(blocklist),
            token_ratelimit: Arc::new(token_ratelimit),
            push,
            identifier,
        };
        module.spawn_blocklist_cleanup();
        module.spawn_token_cleanup();

        Ok(module)
    }

    /// trigger push event got provided token
    #[inline(always)]
    async fn send(&self, token: String) -> PushResult<()> {
        self.push.send(token).await
    }

    fn spawn_blocklist_cleanup(&self) {
        let blocklist = self.blocklist.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                blocklist.cleanup();
            }
        });
    }

    fn spawn_token_cleanup(&self) {
        let token_ratelimit = self.token_ratelimit.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                token_ratelimit.cleanup();
            }
        });
    }

    #[inline(always)]
    pub fn blocklist(&self) -> &Arc<FpushBlocklist> {
        &self.blocklist
    }

    #[inline(always)]
    pub fn ratelimit(&self) -> &Arc<FpushTokenRateLimit> {
        &self.token_ratelimit
    }

    #[inline(always)]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}
