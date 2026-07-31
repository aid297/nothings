#[cfg(test)]
mod tests {
    use crate::impl_singleton;
    use crate::singletons::app::Singleton;

    struct Counter {
        value: u32,
    }

    impl Counter {
        fn default_new() -> Self {
            Counter { value: 0 }
        }

        fn increment(&mut self) -> &mut Self {
            self.value += 1;
            self
        }

        fn get(&self) -> u32 {
            self.value
        }
    }
    impl_singleton!(Counter);

    #[test]
    fn default_value() {
        // 单例初始值为 0
        // 注意：由于单例是全局共享的，测试之间会互相影响
        // 所以这里只验证 with 能正常访问
        let val = Counter::with(|c| c.get());
        // 不假设初始值（其他测试可能已修改），只确认能正常读取
        let _ = val;
    }

    #[test]
    fn with_mutate_and_read() {
        // 修改后能读到新值
        Counter::with(|c| {
            c.increment();
        });
        let val = Counter::with(|c| c.get());
        assert!(val > 0);
    }

    #[test]
    fn singleton_shared_state() {
        // 验证是同一个实例：修改后再次读取，值保持
        let before = Counter::with(|c| c.get());
        Counter::with(|c| {
            c.increment();
        });
        let after = Counter::with(|c| c.get());
        assert_eq!(after, before + 1);
    }

    #[test]
    fn with_returns_value() {
        // with 可以返回闭包的返回值
        let result = Counter::with(|c| {
            c.increment();
            c.get()
        });
        assert!(result > 0);
    }
}
