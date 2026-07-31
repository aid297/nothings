pub(crate) trait Singleton: Sized {
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

#[macro_export]
macro_rules! impl_singleton {
    ($t:ty) => {
        impl $t {
            pub fn instance() -> &'static std::sync::Mutex<$t> {
                static INSTANCE: std::sync::OnceLock<std::sync::Mutex<$t>> = std::sync::OnceLock::new();
                INSTANCE.get_or_init(|| std::sync::Mutex::new(<$t>::default_new()))
            }

            pub fn default() -> Self {
                <$t>::default_new()
            }
        }

        impl $crate::singletons::app::Singleton for $t {
            fn with<F, R>(f: F) -> R
            where
                F: FnOnce(&mut Self) -> R,
            {
                let mut ins = Self::instance().lock().unwrap();
                f(&mut ins)
            }
        }
    };
}

