use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ProtocolError;

/// A single fragment of a larger message that has been split for transport.
///
/// Messages exceeding `max_fragment_size` (default 256 KiB) are split into
/// multiple fragments, each identified by a shared `message_id`, an index,
/// and the total number of fragments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fragment {
    pub message_id: Uuid,
    pub fragment_index: u16,
    pub total_fragments: u16,
    pub data: Vec<u8>,
}

/// Splits large byte payloads into [`Fragment`]s for transport.
///
/// If the data fits within `max_fragment_size`, a single fragment is returned.
pub struct Fragmenter {
    max_fragment_size: usize,
}

impl Fragmenter {
    /// Create a new fragmenter with the given maximum fragment size in bytes.
    pub fn new(max_size: usize) -> Self {
        Self {
            max_fragment_size: max_size,
        }
    }

    /// Check whether the given data exceeds the maximum fragment size.
    pub fn needs_fragmentation(&self, data: &[u8]) -> bool {
        data.len() > self.max_fragment_size
    }

    /// Split data into fragments. If data fits in a single fragment,
    /// returns a `Vec` with one element.
    pub fn fragment(&self, data: &[u8]) -> Vec<Fragment> {
        let message_id = Uuid::new_v4();

        if data.is_empty() {
            return vec![Fragment {
                message_id,
                fragment_index: 0,
                total_fragments: 1,
                data: Vec::new(),
            }];
        }

        let chunks: Vec<&[u8]> = data.chunks(self.max_fragment_size).collect();
        let total_fragments = chunks.len() as u16;

        chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| Fragment {
                message_id,
                fragment_index: i as u16,
                total_fragments,
                data: chunk.to_vec(),
            })
            .collect()
    }
}

/// Tracks pending partial messages and reassembles them once all fragments arrive.
pub struct Reassembler {
    pending: HashMap<Uuid, PendingMessage>,
    timeout_seconds: i64,
}

struct PendingMessage {
    fragments: Vec<Option<Vec<u8>>>,
    total: u16,
    received: u16,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Reassembler {
    /// Create a new reassembler with the given timeout in seconds for partial messages.
    pub fn new(timeout_seconds: i64) -> Self {
        Self {
            pending: HashMap::new(),
            timeout_seconds,
        }
    }

    /// Add a fragment. Returns `Ok(Some(data))` when all fragments have been received
    /// and the full message is reassembled. Returns `Ok(None)` when more fragments
    /// are still expected.
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::FragmentError` if the fragment index is out of bounds
    /// or `total_fragments` is zero.
    pub fn add_fragment(&mut self, fragment: Fragment) -> Result<Option<Vec<u8>>, ProtocolError> {
        if fragment.total_fragments == 0 {
            return Err(ProtocolError::FragmentError(
                "total_fragments must be > 0".to_string(),
            ));
        }

        if fragment.fragment_index >= fragment.total_fragments {
            return Err(ProtocolError::FragmentError(format!(
                "fragment_index {} >= total_fragments {}",
                fragment.fragment_index, fragment.total_fragments
            )));
        }

        let pending = self
            .pending
            .entry(fragment.message_id)
            .or_insert_with(|| PendingMessage {
                fragments: vec![None; fragment.total_fragments as usize],
                total: fragment.total_fragments,
                received: 0,
                created_at: chrono::Utc::now(),
            });

        // Validate total_fragments consistency
        if pending.total != fragment.total_fragments {
            return Err(ProtocolError::FragmentError(format!(
                "Inconsistent total_fragments: expected {}, got {}",
                pending.total, fragment.total_fragments
            )));
        }

        let idx = fragment.fragment_index as usize;

        // Handle duplicate fragments -- don't increment received count
        if pending.fragments[idx].is_none() {
            pending.received += 1;
        }
        pending.fragments[idx] = Some(fragment.data);

        if pending.received == pending.total {
            // All fragments received -- reassemble
            let pending = self.pending.remove(&fragment.message_id).unwrap();
            let mut reassembled = Vec::new();
            for frag_data in pending.fragments {
                reassembled.extend(frag_data.unwrap());
            }
            Ok(Some(reassembled))
        } else {
            Ok(None)
        }
    }

    /// Remove all pending messages that have exceeded the timeout.
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        self.pending.retain(|_id, pending| {
            let elapsed = now.signed_duration_since(pending.created_at).num_seconds();
            elapsed < self.timeout_seconds
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_MAX_SIZE: usize = 262_144; // 256 KiB

    #[test]
    fn small_message_single_fragment() {
        let fragmenter = Fragmenter::new(DEFAULT_MAX_SIZE);
        let data = vec![0xAB; 100];

        assert!(!fragmenter.needs_fragmentation(&data));

        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].fragment_index, 0);
        assert_eq!(fragments[0].total_fragments, 1);
        assert_eq!(fragments[0].data, data);
    }

    #[test]
    fn large_message_splits() {
        let fragmenter = Fragmenter::new(DEFAULT_MAX_SIZE);
        // 600 KB -> ceil(600_000 / 262_144) = 3 fragments
        let data = vec![0xCD; 600_000];

        assert!(fragmenter.needs_fragmentation(&data));

        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 3);

        // All fragments share the same message_id
        let msg_id = fragments[0].message_id;
        for frag in &fragments {
            assert_eq!(frag.message_id, msg_id);
            assert_eq!(frag.total_fragments, 3);
        }

        assert_eq!(fragments[0].fragment_index, 0);
        assert_eq!(fragments[1].fragment_index, 1);
        assert_eq!(fragments[2].fragment_index, 2);

        // First two fragments are exactly max_size
        assert_eq!(fragments[0].data.len(), DEFAULT_MAX_SIZE);
        assert_eq!(fragments[1].data.len(), DEFAULT_MAX_SIZE);
        // Third fragment is the remainder
        assert_eq!(fragments[2].data.len(), 600_000 - 2 * DEFAULT_MAX_SIZE);
    }

    #[test]
    fn fragment_reassemble_round_trip() {
        let fragmenter = Fragmenter::new(DEFAULT_MAX_SIZE);
        let mut reassembler = Reassembler::new(30);

        // 1 MB of data
        let original_data: Vec<u8> = (0..1_000_000u32).map(|i| (i % 256) as u8).collect();

        let fragments = fragmenter.fragment(&original_data);
        assert!(fragments.len() > 1, "Should be multiple fragments");

        let mut result = None;
        for fragment in fragments {
            result = reassembler.add_fragment(fragment).unwrap();
        }

        assert!(result.is_some(), "Should have reassembled data");
        assert_eq!(result.unwrap(), original_data);
    }

    #[test]
    fn fragments_in_order_reassemble() {
        let fragmenter = Fragmenter::new(100); // Small max to force fragmentation
        let mut reassembler = Reassembler::new(30);

        let data = vec![0xFF; 250]; // 3 fragments at max 100

        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 3);

        assert!(reassembler
            .add_fragment(fragments[0].clone())
            .unwrap()
            .is_none());
        assert!(reassembler
            .add_fragment(fragments[1].clone())
            .unwrap()
            .is_none());
        let result = reassembler.add_fragment(fragments[2].clone()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn fragments_out_of_order_reassemble() {
        let fragmenter = Fragmenter::new(100);
        let mut reassembler = Reassembler::new(30);

        let data = vec![0xFF; 250];

        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 3);

        // Add in order: 2, 0, 1
        assert!(reassembler
            .add_fragment(fragments[2].clone())
            .unwrap()
            .is_none());
        assert!(reassembler
            .add_fragment(fragments[0].clone())
            .unwrap()
            .is_none());
        let result = reassembler.add_fragment(fragments[1].clone()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn duplicate_fragment_handled() {
        let fragmenter = Fragmenter::new(100);
        let mut reassembler = Reassembler::new(30);

        let data = vec![0xAA; 200]; // 2 fragments

        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 2);

        // Add fragment 0 twice
        assert!(reassembler
            .add_fragment(fragments[0].clone())
            .unwrap()
            .is_none());
        assert!(reassembler
            .add_fragment(fragments[0].clone())
            .unwrap()
            .is_none());

        // Now add fragment 1 -- should complete
        let result = reassembler.add_fragment(fragments[1].clone()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn partial_fragments_returns_none() {
        let fragmenter = Fragmenter::new(100);
        let mut reassembler = Reassembler::new(30);

        let data = vec![0xBB; 300]; // 3 fragments

        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 3);

        // Only add 2 of 3
        assert!(reassembler
            .add_fragment(fragments[0].clone())
            .unwrap()
            .is_none());
        assert!(reassembler
            .add_fragment(fragments[1].clone())
            .unwrap()
            .is_none());

        // Should not have reassembled yet
        // (No more fragments added, so remains None)
    }

    #[test]
    fn expired_fragments_cleaned_up() {
        let mut reassembler = Reassembler::new(0); // 0 second timeout = immediate expiry

        let fragment = Fragment {
            message_id: Uuid::new_v4(),
            fragment_index: 0,
            total_fragments: 3,
            data: vec![1, 2, 3],
        };

        // Add one fragment (partial message)
        assert!(reassembler
            .add_fragment(fragment.clone())
            .unwrap()
            .is_none());
        assert_eq!(reassembler.pending.len(), 1);

        // Cleanup -- should remove expired
        reassembler.cleanup_expired();
        assert_eq!(reassembler.pending.len(), 0);
    }

    #[test]
    fn empty_data_produces_single_fragment() {
        let fragmenter = Fragmenter::new(DEFAULT_MAX_SIZE);
        let fragments = fragmenter.fragment(&[]);
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].data.len(), 0);
        assert_eq!(fragments[0].total_fragments, 1);
    }

    #[test]
    fn fragment_index_out_of_bounds_returns_error() {
        let mut reassembler = Reassembler::new(30);

        let bad_fragment = Fragment {
            message_id: Uuid::new_v4(),
            fragment_index: 5,
            total_fragments: 3,
            data: vec![1, 2, 3],
        };

        let result = reassembler.add_fragment(bad_fragment);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProtocolError::FragmentError(_)
        ));
    }

    #[test]
    fn zero_total_fragments_returns_error() {
        let mut reassembler = Reassembler::new(30);

        let bad_fragment = Fragment {
            message_id: Uuid::new_v4(),
            fragment_index: 0,
            total_fragments: 0,
            data: vec![],
        };

        let result = reassembler.add_fragment(bad_fragment);
        assert!(result.is_err());
    }

    #[test]
    fn inconsistent_total_fragments_returns_error() {
        let mut reassembler = Reassembler::new(30);
        let msg_id = Uuid::new_v4();

        let frag1 = Fragment {
            message_id: msg_id,
            fragment_index: 0,
            total_fragments: 3,
            data: vec![1],
        };
        let frag2 = Fragment {
            message_id: msg_id,
            fragment_index: 1,
            total_fragments: 5, // Inconsistent!
            data: vec![2],
        };

        assert!(reassembler.add_fragment(frag1).unwrap().is_none());
        let result = reassembler.add_fragment(frag2);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ProtocolError::FragmentError(_)
        ));
    }

    #[test]
    fn exact_boundary_size_no_fragmentation() {
        let fragmenter = Fragmenter::new(100);
        let data = vec![0xCC; 100]; // Exactly max size

        assert!(!fragmenter.needs_fragmentation(&data));
        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].data, data);
    }

    #[test]
    fn one_byte_over_boundary_fragments() {
        let fragmenter = Fragmenter::new(100);
        let data = vec![0xCC; 101]; // One byte over max

        assert!(fragmenter.needs_fragmentation(&data));
        let fragments = fragmenter.fragment(&data);
        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].data.len(), 100);
        assert_eq!(fragments[1].data.len(), 1);
    }

    #[test]
    fn multiple_independent_messages_reassemble_separately() {
        let fragmenter = Fragmenter::new(100);
        let mut reassembler = Reassembler::new(30);

        let data_a = vec![0xAA; 200];
        let data_b = vec![0xBB; 200];

        let frags_a = fragmenter.fragment(&data_a);
        let frags_b = fragmenter.fragment(&data_b);

        // Interleave fragments from both messages
        assert!(reassembler
            .add_fragment(frags_a[0].clone())
            .unwrap()
            .is_none());
        assert!(reassembler
            .add_fragment(frags_b[0].clone())
            .unwrap()
            .is_none());
        let result_a = reassembler.add_fragment(frags_a[1].clone()).unwrap();
        let result_b = reassembler.add_fragment(frags_b[1].clone()).unwrap();

        assert!(result_a.is_some());
        assert!(result_b.is_some());
        assert_eq!(result_a.unwrap(), data_a);
        assert_eq!(result_b.unwrap(), data_b);
    }
}
