use bytey::{ByteBuffer, ByteBufferRead, ByteBufferWrite};
use mmap_bytey::{MByteBuffer, MByteBufferRead, MByteBufferWrite};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::{Postgres, Type};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(pub std::time::Duration);

impl Duration {
    pub fn as_milliseconds(&self) -> u64 {
        self.0.as_millis() as u64
    }

    pub fn from_milliseconds(milliseconds: u64) -> Duration {
        Duration(std::time::Duration::from_millis(milliseconds))
    }

    pub fn as_std(&self) -> std::time::Duration {
        self.0
    }
}

impl From<chrono::Duration> for Duration {
    fn from(duration: chrono::Duration) -> Duration {
        if let Ok(dur) = duration.to_std() {
            Duration(dur)
        } else {
            Duration(std::time::Duration::default())
        }
    }
}

impl AsRef<std::time::Duration> for Duration {
    fn as_ref(&self) -> &std::time::Duration {
        &self.0
    }
}

impl std::ops::Deref for Duration {
    type Target = std::time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl sqlx::Type<Postgres> for Duration {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        *ty == Self::type_info()
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for Duration {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> sqlx::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>>
    {
        let value = <i64 as sqlx::Decode<Postgres>>::decode(value)?;
        let dur = chrono::Duration::try_milliseconds(value)
            .unwrap_or_default()
            .to_std()
            .unwrap_or_default();

        Ok(Self(dur))
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for Duration {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    {
        let chrono_dur = chrono::Duration::from_std(self.0).unwrap_or_default();
        <i64 as sqlx::Encode<Postgres>>::encode(
            chrono_dur.num_milliseconds(),
            buf,
        )
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_milliseconds().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Duration::from_milliseconds(u64::deserialize(deserializer)?))
    }
}

impl ByteBufferRead for Duration {
    fn read_from_bytey_buffer(buffer: &mut ByteBuffer) -> bytey::Result<Self> {
        Ok(Duration::from_milliseconds(buffer.read::<u64>()?))
    }

    fn read_from_bytey_buffer_le(
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<Self> {
        Ok(Duration::from_milliseconds(buffer.read_le::<u64>()?))
    }

    fn read_from_bytey_buffer_be(
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<Self> {
        Ok(Duration::from_milliseconds(buffer.read_be::<u64>()?))
    }
}

impl ByteBufferWrite for &Duration {
    fn write_to_bytey_buffer(
        &self,
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<()> {
        buffer.write(self.as_milliseconds())?;
        Ok(())
    }
    fn write_to_bytey_buffer_le(
        &self,
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<()> {
        buffer.write_le(self.as_milliseconds())?;
        Ok(())
    }
    fn write_to_bytey_buffer_be(
        &self,
        buffer: &mut ByteBuffer,
    ) -> bytey::Result<()> {
        buffer.write_be(self.as_milliseconds())?;
        Ok(())
    }
}

impl MByteBufferRead for Duration {
    fn read_from_mbuffer(buffer: &mut MByteBuffer) -> mmap_bytey::Result<Self> {
        Ok(Duration::from_milliseconds(buffer.read::<u64>()?))
    }

    fn read_from_mbuffer_le(
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<Self> {
        Ok(Duration::from_milliseconds(buffer.read_le::<u64>()?))
    }

    fn read_from_mbuffer_be(
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<Self> {
        Ok(Duration::from_milliseconds(buffer.read_be::<u64>()?))
    }
}

impl MByteBufferWrite for &Duration {
    fn write_to_mbuffer(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write(self.as_milliseconds())?;
        Ok(())
    }
    fn write_to_mbuffer_le(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write_le(self.as_milliseconds())?;
        Ok(())
    }
    fn write_to_mbuffer_be(
        &self,
        buffer: &mut MByteBuffer,
    ) -> mmap_bytey::Result<()> {
        buffer.write_be(self.as_milliseconds())?;
        Ok(())
    }
}
