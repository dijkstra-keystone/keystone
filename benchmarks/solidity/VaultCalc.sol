// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title VaultCalc - Solidity equivalent of stylus-vault for gas comparison
/// @notice Uses same precision (1e18 scale) and algorithms as Keystone Stylus version
contract VaultCalc {
    uint256 public performanceFeeBps;
    uint256 public managementFeeBps;

    uint256 private constant SCALE = 1e18;
    uint256 private constant BPS_DIVISOR = 10000;
    uint256 private constant SECONDS_PER_YEAR = 365 * 24 * 60 * 60;

    /// @notice Calculate shares to mint for deposit (ERC4626 style)
    function calculateSharesForDeposit(
        uint256 assets,
        uint256 totalAssets,
        uint256 totalSupply
    ) external pure returns (uint256) {
        if (assets == 0) return 0;
        if (totalSupply == 0) return assets;
        if (totalAssets == 0) revert("zero total assets");

        return (assets * totalSupply) / totalAssets;
    }

    /// @notice Calculate assets for redemption
    function calculateAssetsForRedeem(
        uint256 shares,
        uint256 totalAssets,
        uint256 totalSupply
    ) external pure returns (uint256) {
        if (shares == 0) return 0;
        if (totalSupply == 0) revert("zero supply");

        return (shares * totalAssets) / totalSupply;
    }

    /// @notice Calculate share price
    function calculateSharePrice(
        uint256 totalAssets,
        uint256 totalSupply
    ) external pure returns (uint256) {
        if (totalSupply == 0) return SCALE;
        return (totalAssets * SCALE) / totalSupply;
    }

    /// @notice Calculate compound yield (iterative)
    function calculateCompoundYield(
        uint256 principal,
        uint256 rateBps,
        uint256 periods
    ) external pure returns (uint256) {
        uint256 rate = (rateBps * SCALE) / BPS_DIVISOR;
        uint256 multiplier = SCALE;

        uint256 n = periods > 365 ? 365 : periods;
        for (uint256 i = 0; i < n; i++) {
            multiplier = (multiplier * (SCALE + rate)) / SCALE;
        }

        return (principal * multiplier) / SCALE;
    }

    /// @notice Calculate APY from APR (iterative)
    function calculateApyFromApr(
        uint256 aprBps,
        uint256 compoundsPerYear
    ) external pure returns (uint256) {
        if (compoundsPerYear == 0) revert("zero compounds");

        uint256 apr = (aprBps * SCALE) / BPS_DIVISOR;
        uint256 ratePerPeriod = apr / compoundsPerYear;

        uint256 n = compoundsPerYear > 365 ? 365 : compoundsPerYear;
        uint256 multiplier = SCALE;

        for (uint256 i = 0; i < n; i++) {
            multiplier = (multiplier * (SCALE + ratePerPeriod)) / SCALE;
        }

        uint256 apy = multiplier - SCALE;
        return (apy * BPS_DIVISOR) / SCALE;
    }

    /// @notice Calculate performance fee
    function calculatePerformanceFee(uint256 gains) external view returns (uint256) {
        if (gains == 0) return 0;
        return (gains * performanceFeeBps) / BPS_DIVISOR;
    }

    /// @notice Calculate management fee
    function calculateManagementFee(
        uint256 totalAssets,
        uint256 secondsElapsed
    ) external view returns (uint256) {
        if (totalAssets == 0 || secondsElapsed == 0) return 0;

        uint256 annualRate = (managementFeeBps * SCALE) / BPS_DIVISOR;
        uint256 timeFraction = (secondsElapsed * SCALE) / SECONDS_PER_YEAR;

        return (totalAssets * annualRate * timeFraction) / (SCALE * SCALE);
    }

    function setPerformanceFee(uint256 feeBps) external {
        performanceFeeBps = feeBps;
    }

    function setManagementFee(uint256 feeBps) external {
        managementFeeBps = feeBps;
    }
}
