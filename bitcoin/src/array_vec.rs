// SPDX-License-Identifier: CC0-1.0

include!("../include/array_vec.rs");

#[cfg(feature = "serde")]
impl<T: Copy + crate::serde::Serialize, const CAP: usize> crate::serde::Serialize
    for ArrayVec<T, CAP>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: crate::serde::Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

#[cfg(feature = "serde")]
impl<'de, T, const CAP: usize> crate::serde::Deserialize<'de> for ArrayVec<T, CAP>
where
    T: Copy + crate::serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::fmt;
        use core::marker::PhantomData;

        use crate::serde::de;

        struct Visitor<T, const CAP: usize>(PhantomData<T>);

        impl<'de, T, const CAP: usize> de::Visitor<'de> for Visitor<T, CAP>
        where
            T: Copy + crate::serde::Deserialize<'de>,
        {
            type Value = ArrayVec<T, CAP>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a sequence of at most {} elements", CAP)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                use de::Error;

                if let Some(hint) = seq.size_hint() {
                    if hint > CAP {
                        return Err(Error::invalid_length(hint, &self));
                    }
                }

                let mut out = ArrayVec::<T, CAP>::new();
                while let Some(elem) = seq.next_element::<T>()? {
                    out.try_push(elem).map_err(|_| Error::invalid_length(out.len() + 1, &self))?;
                }
                Ok(out)
            }
        }
        deserializer.deserialize_seq(Visitor::<T, CAP>(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayVec;

    #[test]
    fn arrayvec_ops() {
        let mut av = ArrayVec::<_, 1>::new();
        assert!(av.is_empty());
        av.push(42);
        assert_eq!(av.len(), 1);
        assert_eq!(av, [42]);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn overflow_push() {
        let mut av = ArrayVec::<_, 0>::new();
        av.push(42);
    }

    #[test]
    #[should_panic(expected = "buffer overflow")]
    fn overflow_extend() {
        let mut av = ArrayVec::<_, 0>::new();
        av.extend_from_slice(&[42]);
    }

    #[test]
    fn extend_from_slice() {
        let mut av = ArrayVec::<u8, 8>::new();
        av.extend_from_slice(b"abc");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_u8() {
        let mut want = ArrayVec::<u8, 8>::new();
        want.extend_from_slice(b"abc");

        let json = serde_json::to_string(&want).expect("serde_json failed to encode");
        let got: ArrayVec<u8, 8> =
            serde_json::from_str(&json).expect("serde_json failed to decode");
        assert_eq!(got, want);

        let bin = bincode::serialize(&want).expect("bincode failed to encode");
        let got: ArrayVec<u8, 8> = bincode::deserialize(&bin).expect("bincode failed to decode");
        assert_eq!(got, want);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_u32() {
        let mut want = ArrayVec::<u32, 4>::new();
        (1..=3).for_each(|i| want.push(i));

        let json = serde_json::to_string(&want).expect("serde_json failed to encode");
        let got: ArrayVec<u32, 4> =
            serde_json::from_str(&json).expect("serde_json failed to decode");
        assert_eq!(got, want);

        let bin = bincode::serialize(&want).expect("bincode failed to encode");
        let got: ArrayVec<u32, 4> = bincode::deserialize(&bin).expect("bincode failed to decode");
        assert_eq!(got, want);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_empty() {
        let want = ArrayVec::<u8, 0>::new();

        let json = serde_json::to_string(&want).expect("serde_json failed to encode");
        assert_eq!(json, "[]");
        let got: ArrayVec<u8, 0> =
            serde_json::from_str(&json).expect("serde_json failed to decode");
        assert_eq!(got, want);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_deserialize_overflow_json_returns_error() {
        // CAP=2 but JSON contains 3 elements -> must error, not panic.
        // Excercises the read-until-overflow path (no usable size_hint).
        let json = "[1,2,3]";
        let res: Result<ArrayVec<u8, 2>, _> = serde_json::from_str(json);
        assert!(res.is_err(), "expected an error for over-capacity input");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_deserialize_overflow_bincode_returns_error() {
        // Exercises the size_hint > CAP fast-reject path; bincode prefixes the
        // sequence with a length, which becomes the sze_hint on deserialize.
        let slice: &[u8] = &[1, 2, 3];
        let bin = bincode::serialize(slice).expect("bincode failed to encode");
        let res: Result<ArrayVec<u8, 2>, _> = bincode::deserialize(&bin);
        assert!(res.is_err(), "expected an error for over-capacity input");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_matches_vec_wire_format() {
        // Verifies the on-the-wire encoding is identical to `Vec<T>`/`&[T]` so
        // that an `ArrayVec<T, CAP>` is interchangeable with `Vec<T>` in serde.
        let slice: &[u8] = &[1, 2, 3];
        let want = ArrayVec::<u8, 8>::from_slice(slice);

        // JSON
        let av_json = serde_json::to_string(&want).expect("serde_json failed to encode");
        let slice_json = serde_json::to_string(slice).expect("serde_json failed to encode");
        assert_eq!(av_json, slice_json);

        // Bincode.
        let av_bin = bincode::serialize(&want).expect("bincode failed to encode");
        let slice_bin = bincode::serialize(slice).expect("bincode failed to encode");
        assert_eq!(av_bin, slice_bin);

        // Deserialize the slice-encoded bytes into ArrayVec.
        let got: ArrayVec<u8, 8> =
            serde_json::from_str(&slice_json).expect("serde_json failed to decode");
        assert_eq!(got, want);

        let got: ArrayVec<u8, 8> =
            bincode::deserialize(&slice_bin).expect("bincode failed to decode");
        assert_eq!(got, want);
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::unwind(16)] // One greater than 15 (max number of elements).
    #[kani::proof]
    fn no_out_of_bounds_less_than_cap() {
        const CAP: usize = 32;
        let n = kani::any::<u32>();
        let elements = (n & 0x0F) as usize; // Just use 4 bits.

        let val = kani::any::<u32>();

        let mut v = ArrayVec::<u32, CAP>::new();
        for _ in 0..elements {
            v.push(val);
        }

        for i in 0..elements {
            assert_eq!(v[i], val);
        }
    }

    #[kani::unwind(16)] // One greater than 15.
    #[kani::proof]
    fn no_out_of_bounds_upto_cap() {
        const CAP: usize = 15;
        let elements = CAP;

        let val = kani::any::<u32>();

        let mut v = ArrayVec::<u32, CAP>::new();
        for _ in 0..elements {
            v.push(val);
        }

        for i in 0..elements {
            assert_eq!(v[i], val);
        }
    }
}
