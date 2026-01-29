// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/SolidityBenchmark.sol";

contract GasBenchmarkTest is Test {
    SolidityBenchmark public benchmark;

    uint256 constant SCALE = 1e18;

    function setUp() public {
        benchmark = new SolidityBenchmark();
    }

    // ========================================================================
    // Lending Gas Benchmarks
    // ========================================================================

    function test_gas_calculateHealthFactor() public view {
        uint256 collateral = 10_000 * SCALE;
        uint256 debt = 5_000 * SCALE;
        uint256 threshold = 8000; // 80%

        uint256 gasBefore = gasleft();
        benchmark.calculateHealthFactor(collateral, debt, threshold);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateHealthFactor gas:", gasUsed);
    }

    function test_gas_calculateLiquidationPrice() public view {
        uint256 collateralAmount = 10 * SCALE;
        uint256 debt = 8_000 * SCALE;
        uint256 threshold = 8000;

        uint256 gasBefore = gasleft();
        benchmark.calculateLiquidationPrice(collateralAmount, debt, threshold);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateLiquidationPrice gas:", gasUsed);
    }

    function test_gas_calculateMaxBorrow() public view {
        uint256 collateral = 10_000 * SCALE;
        uint256 targetHF = 15 * SCALE / 10; // 1.5
        uint256 threshold = 8000;

        uint256 gasBefore = gasleft();
        benchmark.calculateMaxBorrow(collateral, targetHF, threshold);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateMaxBorrow gas:", gasUsed);
    }

    // ========================================================================
    // AMM Gas Benchmarks
    // ========================================================================

    function test_gas_calculateSwapOutput() public view {
        uint256 reserveIn = 1_000_000 * SCALE;
        uint256 reserveOut = 1_000_000 * SCALE;
        uint256 amountIn = 10_000 * SCALE;
        uint256 feeBps = 30;

        uint256 gasBefore = gasleft();
        benchmark.calculateSwapOutput(reserveIn, reserveOut, amountIn, feeBps);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateSwapOutput gas:", gasUsed);
    }

    function test_gas_calculatePriceImpact() public view {
        uint256 reserveIn = 1_000_000 * SCALE;
        uint256 reserveOut = 1_000_000 * SCALE;
        uint256 amountIn = 100_000 * SCALE;
        uint256 feeBps = 30;

        uint256 gasBefore = gasleft();
        benchmark.calculatePriceImpact(reserveIn, reserveOut, amountIn, feeBps);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculatePriceImpact gas:", gasUsed);
    }

    function test_gas_calculateSpotPrice() public view {
        uint256 reserveA = 2_000 * SCALE;
        uint256 reserveB = 4_000 * SCALE;

        uint256 gasBefore = gasleft();
        benchmark.calculateSpotPrice(reserveA, reserveB);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateSpotPrice gas:", gasUsed);
    }

    // ========================================================================
    // Vault Gas Benchmarks
    // ========================================================================

    function test_gas_calculateSharesForDeposit() public view {
        uint256 assets = 100 * SCALE;
        uint256 totalAssets = 1_000 * SCALE;
        uint256 totalSupply = 500 * SCALE;

        uint256 gasBefore = gasleft();
        benchmark.calculateSharesForDeposit(assets, totalAssets, totalSupply);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateSharesForDeposit gas:", gasUsed);
    }

    function test_gas_calculateAssetsForRedeem() public view {
        uint256 shares = 50 * SCALE;
        uint256 totalAssets = 1_000 * SCALE;
        uint256 totalSupply = 500 * SCALE;

        uint256 gasBefore = gasleft();
        benchmark.calculateAssetsForRedeem(shares, totalAssets, totalSupply);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateAssetsForRedeem gas:", gasUsed);
    }

    function test_gas_calculateSharePrice() public view {
        uint256 totalAssets = 2_000 * SCALE;
        uint256 totalSupply = 1_000 * SCALE;

        uint256 gasBefore = gasleft();
        benchmark.calculateSharePrice(totalAssets, totalSupply);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateSharePrice gas:", gasUsed);
    }

    function test_gas_calculateCompoundYield_12periods() public view {
        uint256 principal = 1_000 * SCALE;
        uint256 rateBps = 100; // 1%
        uint256 periods = 12;

        uint256 gasBefore = gasleft();
        benchmark.calculateCompoundYield(principal, rateBps, periods);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateCompoundYield (12 periods) gas:", gasUsed);
    }

    function test_gas_calculateCompoundYield_365periods() public view {
        uint256 principal = 1_000 * SCALE;
        uint256 rateBps = 10; // 0.1%
        uint256 periods = 365;

        uint256 gasBefore = gasleft();
        benchmark.calculateCompoundYield(principal, rateBps, periods);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateCompoundYield (365 periods) gas:", gasUsed);
    }

    function test_gas_calculateApyFromApr() public view {
        uint256 aprBps = 1000; // 10%
        uint256 compounds = 12;

        uint256 gasBefore = gasleft();
        benchmark.calculateApyFromApr(aprBps, compounds);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("calculateApyFromApr gas:", gasUsed);
    }

    // ========================================================================
    // Batch Operations (more realistic scenarios)
    // ========================================================================

    function test_gas_batchHealthFactorCheck() public view {
        uint256 gasBefore = gasleft();

        // Simulate checking 10 positions
        for (uint256 i = 0; i < 10; i++) {
            uint256 collateral = (1000 + i * 100) * SCALE;
            uint256 debt = (500 + i * 50) * SCALE;
            benchmark.calculateHealthFactor(collateral, debt, 8000);
        }

        uint256 gasUsed = gasBefore - gasleft();
        console.log("batchHealthFactorCheck (10 positions) gas:", gasUsed);
        console.log("per position:", gasUsed / 10);
    }

    function test_gas_batchSwapQuotes() public view {
        uint256 reserveIn = 1_000_000 * SCALE;
        uint256 reserveOut = 1_000_000 * SCALE;

        uint256 gasBefore = gasleft();

        // Simulate quoting 10 different amounts
        for (uint256 i = 1; i <= 10; i++) {
            uint256 amountIn = i * 1000 * SCALE;
            benchmark.calculateSwapOutput(reserveIn, reserveOut, amountIn, 30);
        }

        uint256 gasUsed = gasBefore - gasleft();
        console.log("batchSwapQuotes (10 quotes) gas:", gasUsed);
        console.log("per quote:", gasUsed / 10);
    }
}
