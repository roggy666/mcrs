use std::io::Write;

use mcrs_minecraft_protocol::{Decode, Encode, Result, VarInt};
use mcrs_minecraft_resource::ResourceLocationBuf;

const INLINE: i32 = 0;

#[derive(Clone, Debug, PartialEq)]
pub enum SoundHolder {
    Registry(u16),
    Inline {
        name: ResourceLocationBuf,
        range: Option<f32>,
    },
}

impl Encode for SoundHolder {
    fn encode(&self, mut w: impl Write) -> Result<()> {
        match self {
            Self::Registry(id) => VarInt(i32::from(*id).saturating_add(1)).encode(w),
            Self::Inline { name, range } => {
                VarInt(INLINE).encode(&mut w)?;
                name.encode(&mut w)?;
                range.is_some().encode(&mut w)?;
                if let Some(range) = range {
                    range.encode(&mut w)?;
                }
                Ok(())
            }
        }
    }
}

impl Decode<'_> for SoundHolder {
    fn decode(r: &mut &[u8]) -> Result<Self> {
        let form = VarInt::decode(r)?.0;
        if form == INLINE {
            let name = ResourceLocationBuf::decode(r)?;
            let range = if bool::decode(r)? {
                Some(f32::decode(r)?)
            } else {
                None
            };
            return Ok(Self::Inline { name, range });
        }
        let id = form
            .checked_sub(1)
            .and_then(|id| u16::try_from(id).ok())
            .ok_or_else(|| {
                mcrs_minecraft_protocol::Error::invalid("a sound holder id is out of range")
            })?;
        Ok(Self::Registry(id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundSource {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}

impl SoundSource {
    const ALL: [Self; 10] = [
        Self::Master,
        Self::Music,
        Self::Records,
        Self::Weather,
        Self::Blocks,
        Self::Hostile,
        Self::Neutral,
        Self::Players,
        Self::Ambient,
        Self::Voice,
    ];

    #[must_use]
    pub fn index(self) -> i32 {
        Self::ALL
            .iter()
            .position(|source| *source == self)
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(0)
    }
}

impl Encode for SoundSource {
    fn encode(&self, w: impl Write) -> Result<()> {
        VarInt(self.index()).encode(w)
    }
}

impl Decode<'_> for SoundSource {
    fn decode(r: &mut &[u8]) -> Result<Self> {
        let index = VarInt::decode(r)?.0;
        usize::try_from(index)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
            .ok_or_else(|| {
                mcrs_minecraft_protocol::Error::invalid("a sound source is out of range")
            })
    }
}
