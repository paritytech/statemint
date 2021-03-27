pub mod currency {
    use node_primitives::Balance;

    pub const DOTS: Balance = 1_000_000_000_000;
	pub const DOLLARS: Balance = DOTS / 100;       // 10_000_000_000
	pub const CENTS: Balance = DOLLARS / 100;      // 100_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
        // 1/10 of Polkadot v29
		items as Balance * 2 * DOLLARS + (bytes as Balance) * 10 * MILLICENTS
	}
}

/// Fee-related.
pub mod fee {
	pub use sp_runtime::Perbill;
	use node_primitives::Balance;
	// use ExtrinsicBaseWeight;
	use frame_support::weights::{
        WeightToFeePolynomial, WeightToFeeCoefficient, WeightToFeeCoefficients,
        constants::ExtrinsicBaseWeight,
    };
	use smallvec::smallvec;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			let p = super::currency::CENTS;
			let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}
