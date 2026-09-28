use super::{RcMode, RcModeLogic, RxChannel, RxChannelRange, RxChannels};
use core::ops::{Index, IndexMut, Range, RangeTo};

#[cfg(feature = "storage")]
use sequential_storage::map::PostcardValue;
#[cfg(feature = "serde")]
use {
    postcard::experimental::max_size::MaxSize,
    serde::{Deserialize, Serialize},
};

/// Mode Activation Condition (MAC).<br><br>
///
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct ModeActivationCondition {
    pub mode_id: RcMode,
    pub mode_logic: RcModeLogic,
    pub channel: RxChannel,
    pub range: RxChannelRange,
    pub linked_to: u8,
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for ModeActivationCondition {}

impl Default for ModeActivationCondition {
    fn default() -> Self {
        Self::new(RcMode::Arm)
    }
}

impl ModeActivationCondition {
    /// Constructor.
    #[must_use]
    pub const fn new(mode_id: RcMode) -> Self {
        Self {
            mode_id,
            mode_logic: RcModeLogic::Or,
            channel: RxChannel::Aux1,
            range: RxChannelRange::new(),
            linked_to: 0,
        }
    }
    /// Set the channel of a newly constructed MAC.
    #[must_use]
    pub const fn with_channel(mut self, channel: RxChannel) -> Self {
        self.channel = channel;
        self
    }
    /// Set the range of a newly constructed MAC.
    #[must_use]
    pub const fn with_range(mut self, range: RxChannelRange) -> Self {
        self.range = range;
        self
    }
    /// Set the mode id of a newly constructed MAC.
    #[must_use]
    pub const fn with_mode_id(mut self, mode_id: RcMode) -> Self {
        self.mode_id = mode_id;
        self
    }
    /// Set the mode logic of a newly constructed MAC.
    #[must_use]
    pub const fn with_mode_logic(mut self, mode_logic: RcModeLogic) -> Self {
        self.mode_logic = mode_logic;
        self
    }
    /// Set the linked to of a newly constructed MAC.
    #[must_use]
    pub const fn with_linked_to(mut self, linked_to: u8) -> Self {
        self.linked_to = linked_to;
        self
    }
}

impl ModeActivationCondition {
    /// Sets `range`, `mode_id`, and `aux_channel_index`.
    pub fn set(&mut self, range: RxChannelRange, mode_id: RcMode, channel: RxChannel) {
        self.range = range;
        self.mode_id = mode_id;
        self.channel = channel;
    }
    #[must_use]
    #[inline]
    pub fn is_active(&self, channels: &RxChannels) -> bool {
        self.range.is_active(channels, self.channel)
    }
}

/// Array of mode activation conditions.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, MaxSize))]
pub struct ModeActivationConditions([Option<ModeActivationCondition>; Self::COUNT]);

impl ModeActivationConditions {
    pub const COUNT: usize = 20;

    #[must_use]
    pub const fn new() -> Self {
        Self([None; Self::COUNT])
    }

    /// Appends an item to the first available `None` slot.
    /// This function is **O(N)**, but that is fine, since MACs are only added at initialization, not in RX task loop.
    /// # Errors
    /// Returns `Ok(usize)` with the slot index on success, or `Err(ModeActivationCondition)` if full.
    pub fn push(&mut self, mac: ModeActivationCondition) -> Result<u8, ModeActivationCondition> {
        for (ii, slot) in self.0.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(mac);
                #[allow(clippy::cast_possible_truncation)]
                return Ok(ii as u8);
            }
        }
        // Return the condition back to the caller if there's no room
        Err(mac)
    }
    /*/// Appends an item to the first available slot.
    /// If the container is full, the item is silently ignored.
    #[inline]
    pub fn push_if_space_available(&mut self, mac: ModeActivationCondition) {
        let _ = self.push(mac);
    }*/
}

#[cfg(feature = "storage")]
impl PostcardValue<'_> for ModeActivationConditions {}

impl Default for ModeActivationConditions {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for ModeActivationConditions {
    type Output = Option<ModeActivationCondition>;

    #[inline]
    fn index(&self, index: usize) -> &Option<ModeActivationCondition> {
        &self.0[index]
    }
}

impl Index<RangeTo<usize>> for ModeActivationConditions {
    type Output = [Option<ModeActivationCondition>];

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &[Option<ModeActivationCondition>] {
        &self.0[index]
    }
}

impl IntoIterator for ModeActivationConditions {
    type Item = Option<ModeActivationCondition>;
    type IntoIter = core::array::IntoIter<Option<ModeActivationCondition>, { Self::COUNT }>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IndexMut<usize> for ModeActivationConditions {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Option<ModeActivationCondition> {
        &mut self.0[index]
    }
}

impl Index<Range<usize>> for ModeActivationConditions {
    type Output = [Option<ModeActivationCondition>];

    #[inline]
    fn index(&self, index: Range<usize>) -> &[Option<ModeActivationCondition>] {
        &self.0[index]
    }
}

impl IndexMut<Range<usize>> for ModeActivationConditions {
    #[inline]
    fn index_mut(&mut self, index: Range<usize>) -> &mut [Option<ModeActivationCondition>] {
        &mut self.0[index]
    }
}

#[cfg(test)]
mod test_traits {
    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_serde<T: Serialize + MaxSize + for<'a> Deserialize<'a>>() {}
    #[cfg(feature = "storage")]
    fn is_storage<T: for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<ModeActivationCondition>();
    }
    #[cfg(feature = "serde")]
    #[test]
    fn serde_types() {
        is_serde::<ModeActivationCondition>();
    }
    #[cfg(feature = "storage")]
    #[test]
    fn storage_types() {
        is_storage::<ModeActivationCondition>();
    }
}
