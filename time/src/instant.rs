use crate::Duration;
#[cfg(feature = "enable_bytey")]
use bytey::{ByteBuffer, ByteBufferRead, ByteBufferWrite};
#[cfg(feature = "enable_mmap_bytey")]
use mmap_bytey::{MByteBuffer, MByteBufferRead, MByteBufferWrite};
#[cfg(feature = "enable_serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(all(feature = "enable_sqlx", feature = "sqlx_postgres"))]
use sqlx::{Postgres, Type};
use std::ops::*;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// A Instant Struct containing milliseconds since first call as a u64.
/// If you use recent() please ensure you are calling now() at least once
/// before calling recent() or use the updater which will call now() for you
/// keeping recent() uptodate.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(pub u64);

// We will hold the entire Duration for Rebuild.
static RECENT: AtomicU64 = AtomicU64::new(0);
static INSTANT: LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(|| std::time::Instant::now());

impl Instant {
    /// Returns an instant corresponding to "now".
    /// Default Now Incredibly Slow use the recent function with the Updater enabled instead.
    /// This will Avoid Slowdowns due to system calls.
    ///
    pub fn now() -> Instant {
        let dur = Self::_now();
        Self::_update(dur);
        Instant(dur)
    }

    /// Updates the stored Instance internals for recent() usage.
    pub fn update() {
        let dur = Self::_now();
        Self::_update(dur);
    }

    /// Returns an instant corresponding to the updaters last updated Recent secs and nanosecs.
    /// Much faster than now but needs the updater to be running for it to work correctly.
    ///
    pub fn recent() -> Instant {
        Instant(Self::_recent())
    }

    pub fn to_dur(self) -> i64 {
        let mut dur: i64 = 0;

        if let Ok(approx) = chrono::Duration::from_std(
            Duration::from_milliseconds(
                self.0.saturating_sub(Instant::recent().0),
            )
            .as_std(),
        ) && approx
            > chrono::Duration::try_milliseconds(1).unwrap_or_default()
        {
            dur = approx.num_milliseconds();
        }

        dur
    }

    pub fn from_dur(dur: i64) -> Instant {
        let duration =
            chrono::Duration::try_milliseconds(dur).unwrap_or_default();
        let mut instant_now = Instant::recent();

        if let Ok(dur) = duration.to_std() {
            instant_now.0 += dur.as_millis() as u64;
        }

        instant_now
    }

    pub fn duration_since(&self, instant: Instant) -> Duration {
        let offset = self.0.saturating_sub(instant.0);

        Duration::from_milliseconds(offset)
    }

    #[inline]
    fn _now() -> u64 {
        let now: std::time::Instant = std::time::Instant::now();
        now.duration_since(*INSTANT).as_millis() as u64
    }

    #[inline]
    fn _recent() -> u64 {
        let recent = RECENT.load(Ordering::Relaxed);

        if recent != 0 {
            recent
        } else {
            let now = Self::_now();
            Self::_update(now);
            Self::_recent()
        }
    }

    #[inline]
    fn _update(millsecs: u64) {
        RECENT.store(millsecs, Ordering::Relaxed);
    }
}

impl std::ops::Deref for Instant {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(all(feature = "enable_sqlx", feature = "sqlx_postgres"))]
impl sqlx::Type<Postgres> for Instant {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        *ty == Self::type_info()
    }
}

#[cfg(all(feature = "enable_sqlx", feature = "sqlx_postgres"))]
impl<'r> sqlx::Decode<'r, Postgres> for Instant {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> sqlx::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>>
    {
        let value = <i64 as sqlx::Decode<Postgres>>::decode(value)?;
        Ok(Self::from_dur(value))
    }
}

#[cfg(all(feature = "enable_sqlx", feature = "sqlx_postgres"))]
impl<'q> sqlx::Encode<'q, Postgres> for Instant {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    {
        <i64 as sqlx::Encode<Postgres>>::encode(self.to_dur(), buf)
    }
}

#[cfg(feature = "enable_serde")]
impl Serialize for Instant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_dur().serialize(serializer)
    }
}

#[cfg(feature = "enable_serde")]
impl<'de> Deserialize<'de> for Instant {
    fn deserialize<D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Instant::from_dur(i64::deserialize(deserializer)?))
    }
}

#[cfg(feature = "enable_bytey")]
impl ByteBufferRead for Instant {
    fn read_from_bytey_buffer(buffer: &mut ByteBuffer) -> bytey::Result<Self> {
        Ok(Instant::from_dur(buffer.read::<i64>()?))
    }

    fn read_from_bytey_buffer_le(
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<Self> {
        Ok(Instant::from_dur(buffer.read_le::<i64>()?))
    }

    fn read_from_bytey_buffer_be(
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<Self> {
        Ok(Instant::from_dur(buffer.read_be::<i64>()?))
    }
}

#[cfg(feature = "enable_bytey")]
impl ByteBufferWrite for &Instant {
    fn write_to_bytey_buffer(
        &self,
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<()> {
        buffer.write(self.to_dur())?;
        Ok(())
    }
    fn write_to_bytey_buffer_le(
        &self,
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<()> {
        buffer.write_le(self.to_dur())?;
        Ok(())
    }
    fn write_to_bytey_buffer_be(
        &self,
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<()> {
        buffer.write_be(self.to_dur())?;
        Ok(())
    }
}

#[cfg(feature = "enable_mmap_bytey")]
impl MByteBufferRead for Instant {
    fn read_from_mbuffer(buffer: &mut MByteBuffer) -> mmap_bytey::Result<Self> {
        Ok(Instant::from_dur(buffer.read::<i64>()?))
    }

    fn read_from_mbuffer_le(
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<Self> {
        Ok(Instant::from_dur(buffer.read_le::<i64>()?))
    }

    fn read_from_mbuffer_be(
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<Self> {
        Ok(Instant::from_dur(buffer.read_be::<i64>()?))
    }
}

#[cfg(feature = "enable_mmap_bytey")]
impl MByteBufferWrite for &Instant {
    fn write_to_mbuffer(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write(self.to_dur())?;
        Ok(())
    }
    fn write_to_mbuffer_le(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write_le(self.to_dur())?;
        Ok(())
    }
    fn write_to_mbuffer_be(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write_be(self.to_dur())?;
        Ok(())
    }
}

#[cfg(feature = "enable_mmap_bytey")]
impl MByteBufferWrite for Instant {
    fn write_to_mbuffer(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write(self.to_dur())?;
        Ok(())
    }
    fn write_to_mbuffer_le(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write_le(self.to_dur())?;
        Ok(())
    }
    fn write_to_mbuffer_be(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write_be(self.to_dur())?;
        Ok(())
    }
}

impl Add<chrono::Duration> for Instant {
    type Output = Instant;

    fn add(self, other: chrono::Duration) -> Instant {
        if let Ok(dur) = other.to_std() {
            Instant(self.0 + dur.as_millis() as u64)
        } else {
            Instant(self.0)
        }
    }
}

impl Add<std::time::Duration> for Instant {
    type Output = Instant;

    fn add(self, other: std::time::Duration) -> Instant {
        Instant(self.0 + other.as_millis() as u64)
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, other: Instant) -> Duration {
        Duration::from_milliseconds(self.0.saturating_sub(other.0))
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn sub(self, rhs: Duration) -> Instant {
        Instant(self.0 - rhs.as_milliseconds())
    }
}

impl SubAssign<Duration> for Instant {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn add(self, rhs: Duration) -> Instant {
        Instant(self.0 + rhs.as_milliseconds())
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Default for Instant {
    fn default() -> Instant {
        Self::now()
    }
}
