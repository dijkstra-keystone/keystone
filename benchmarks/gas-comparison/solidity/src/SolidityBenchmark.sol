// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title Solidity Benchmark - Fixed-point math comparison baseline
/// @notice Implements the same calculations as Keystone Stylus contracts
/// @dev Uses Solidity's native uint256 math with 18 decimal precision

contract SolidityBenchmark {
    uint256 public constant SCALE = 1e18;
    uint256 public constant BPS_DIVISOR = 10000;

    // ========================================================================
    // Lending Calculations (compare with stylus-lending)
    // ========================================================================

    /// @notice Calculate health factor: (collateral * threshold) / debt
    /// @param collateralValue Collateral value scaled by 1e18
    /// @param debtValue Debt value scaled by 1e18
    /// @param thresholdBps Liquidation threshold in basis points (e.g., 8000 = 80%)
    /// @return Health factor scaled by 1e18
    function calculateHealthFactor(
        uint256 collateralValue,
        uint256 debtValue,
        uint256 thresholdBps
    ) external pure returns (uint256) {
        if (debtValue == 0) return type(uint256).max;

        uint256 threshold = (thresholdBps * SCALE) / BPS_DIVISOR;
        uint256 weightedCollateral = (collateralValue * threshold) / SCALE;
        return (weightedCollateral * SCALE) / debtValue;
    }

    /// @notice Calculate liquidation price
    /// @param collateralAmount Amount of collateral tokens
    /// @param debtValue Total debt value
    /// @param thresholdBps Liquidation threshold in basis points
    /// @return Liquidation price scaled by 1e18
    function calculateLiquidationPrice(
        uint256 collateralAmount,
        uint256 debtValue,
        uint256 thresholdBps
    ) external pure returns (uint256) {
        require(collateralAmount > 0, "zero collateral");

        uint256 threshold = (thresholdBps * SCALE) / BPS_DIVISOR;
        uint256 denominator = (collateralAmount * threshold) / SCALE;
        return (debtValue * SCALE) / denominator;
    }

    /// @notice Calculate maximum borrowable amount
    /// @param collateralValue Collateral value
    /// @param targetHealthFactor Target health factor (e.g., 1.5e18)
    /// @param thresholdBps Liquidation threshold in basis points
    /// @return Maximum borrow amount
    function calculateMaxBorrow(
        uint256 collateralValue,
        uint256 targetHealthFactor,
        uint256 thresholdBps
    ) external pure returns (uint256) {
        uint256 threshold = (thresholdBps * SCALE) / BPS_DIVISOR;
        uint256 weighted = (collateralValue * threshold) / SCALE;
        return (weighted * SCALE) / targetHealthFactor;
    }

    // ========================================================================
    // AMM Calculations (compare with stylus-amm)
    // ========================================================================

    /// @notice Calculate swap output for constant product AMM
    /// @param reserveIn Input token reserve
    /// @param reserveOut Output token reserve
    /// @param amountIn Input amount
    /// @param feeBps Fee in basis points (e.g., 30 = 0.3%)
    /// @return Output amount
    function calculateSwapOutput(
        uint256 reserveIn,
        uint256 reserveOut,
        uint256 amountIn,
        uint256 feeBps
    ) external pure returns (uint256) {
        require(reserveIn > 0 && reserveOut > 0, "zero reserve");
        if (amountIn == 0) return 0;

        uint256 feeMultiplier = BPS_DIVISOR - feeBps;
        uint256 amountInWithFee = (amountIn * feeMultiplier) / BPS_DIVISOR;

        uint256 numerator = reserveOut * amountInWithFee;
        uint256 denominator = reserveIn + amountInWithFee;

        return numerator / denominator;
    }

    /// @notice Calculate price impact
    /// @param reserveIn Input reserve
    /// @param reserveOut Output reserve
    /// @param amountIn Input amount
    /// @param feeBps Fee in basis points
    /// @return Price impact scaled by 1e18 (1e16 = 1%)
    function calculatePriceImpact(
        uint256 reserveIn,
        uint256 reserveOut,
        uint256 amountIn,
        uint256 feeBps
    ) external pure returns (uint256) {
        if (amountIn == 0) return 0;

        uint256 spotPrice = (reserveOut * SCALE) / reserveIn;

        uint256 feeMultiplier = BPS_DIVISOR - feeBps;
        uint256 amountInWithFee = (amountIn * feeMultiplier) / BPS_DIVISOR;
        uint256 numerator = reserveOut * amountInWithFee;
        uint256 denominator = reserveIn + amountInWithFee;
        uint256 amountOut = numerator / denominator;

        uint256 effectivePrice = (amountOut * SCALE) / amountIn;

        if (effectivePrice >= spotPrice) return 0;
        return ((spotPrice - effectivePrice) * SCALE) / spotPrice;
    }

    /// @notice Calculate spot price
    /// @param reserveA Reserve of token A
    /// @param reserveB Reserve of token B
    /// @return Price of A in terms of B, scaled by 1e18
    function calculateSpotPrice(
        uint256 reserveA,
        uint256 reserveB
    ) external pure returns (uint256) {
        require(reserveA > 0, "zero reserve");
        return (reserveB * SCALE) / reserveA;
    }

    // ========================================================================
    // Vault Calculations (compare with stylus-vault)
    // ========================================================================

    /// @notice Calculate shares for deposit (ERC4626)
    /// @param assets Amount of assets to deposit
    /// @param totalAssets Current total assets in vault
    /// @param totalSupply Current total share supply
    /// @return Shares to mint
    function calculateSharesForDeposit(
        uint256 assets,
        uint256 totalAssets,
        uint256 totalSupply
    ) external pure returns (uint256) {
        if (assets == 0) return 0;
        if (totalSupply == 0) return assets;
        require(totalAssets > 0, "zero total assets");

        return (assets * totalSupply) / totalAssets;
    }

    /// @notice Calculate assets for redemption (ERC4626)
    /// @param shares Shares to redeem
    /// @param totalAssets Current total assets
    /// @param totalSupply Current total supply
    /// @return Assets to return
    function calculateAssetsForRedeem(
        uint256 shares,
        uint256 totalAssets,
        uint256 totalSupply
    ) external pure returns (uint256) {
        if (shares == 0) return 0;
        require(totalSupply > 0, "zero supply");

        return (shares * totalAssets) / totalSupply;
    }

    /// @notice Calculate share price
    /// @param totalAssets Total assets in vault
    /// @param totalSupply Total share supply
    /// @return Price per share scaled by 1e18
    function calculateSharePrice(
        uint256 totalAssets,
        uint256 totalSupply
    ) external pure returns (uint256) {
        if (totalSupply == 0) return SCALE;
        return (totalAssets * SCALE) / totalSupply;
    }

    /// @notice Calculate compound yield
    /// @param principal Initial amount
    /// @param rateBps Rate per period in basis points
    /// @param periods Number of compounding periods
    /// @return Final amount after compounding
    function calculateCompoundYield(
        uint256 principal,
        uint256 rateBps,
        uint256 periods
    ) external pure returns (uint256) {
        uint256 rate = (rateBps * SCALE) / BPS_DIVISOR;
        uint256 onePlusRate = SCALE + rate;

        uint256 multiplier = SCALE;
        for (uint256 i = 0; i < periods && i < 365; i++) {
            multiplier = (multiplier * onePlusRate) / SCALE;
        }

        return (principal * multiplier) / SCALE;
    }

    /// @notice Calculate APY from APR
    /// @param aprBps Annual rate in basis points
    /// @param compoundsPerYear Compounding frequency
    /// @return APY in basis points scaled by 1e18
    function calculateApyFromApr(
        uint256 aprBps,
        uint256 compoundsPerYear
    ) external pure returns (uint256) {
        require(compoundsPerYear > 0, "zero compounds");

        uint256 apr = (aprBps * SCALE) / BPS_DIVISOR;
        uint256 ratePerPeriod = apr / compoundsPerYear;
        uint256 onePlusRate = SCALE + ratePerPeriod;

        uint256 multiplier = SCALE;
        uint256 n = compoundsPerYear > 365 ? 365 : compoundsPerYear;
        for (uint256 i = 0; i < n; i++) {
            multiplier = (multiplier * onePlusRate) / SCALE;
        }

        uint256 apy = multiplier - SCALE;
        return (apy * BPS_DIVISOR) / SCALE;
    }
}
