#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathError {
    // --- General Arithmetic ---
    /// Denominator in a division is zero.
    ZeroDenominator,
    /// Multiplication or addition exceeded the maximum value of u128 or U256.
    Overflow,
    /// Subtraction resulted in a negative number for an unsigned integer.
    Underflow,

    // --- Tick Math Errors ---
    /// The provided tick is below the minimum allowed tick (e.g., < -443636).
    TickOutOfBounds,
    /// The provided sqrt_price is 0 or exceeds the maximum allowed Q64.64 price.
    SqrtPriceOutOfBounds,
    /// The provided tick lower bound is greater than or equal to the tick upper bound.
    InvalidTickRange,

    /// sqrt price is zero
    ZeroSqrtPrice,

    // --- Liquidity Math Errors ---
    /// Attempted to calculate amounts for a position with zero liquidity.
    ZeroLiquidity,
    /// The lower sqrt price is greater than or equal to the upper sqrt price.
    InvalidSqrtPriceRange,

    // --- Swap Math Errors (Preparing for the Final Boss) ---
    /// The user provided 0 as the amount to swap.
    ZeroAmountSpecified,
    /// The swap cannot proceed because the pool has no liquidity in the current range.
    InsufficientLiquidity,
    /// The swap pushed the price to the exact limit set by the user (expected behavior, not a crash).
    PriceLimitReached,
}
