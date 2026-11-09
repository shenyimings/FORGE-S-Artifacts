# Introduction

We express our gratitude to the Truth Network team for the collaborative engagement that enabled the execution of this Blockchain Protocol Security Assessment.

Truth Network integrates prediction markets into media platforms through their Truth Protocol blockchain system, allowing audiences to make predictions on future outcomes across sports, politics, finance, and entertainment. Truth Protocol is a Substrate-based blockchain infrastructure (forked from Zeitgeist) that provides the foundational layer for creating, managing, and resolving prediction markets with transparency and security. The system handles market creation, oracle reporting, token economics, dispute resolution, and regulatory compliance, serving as the trust layer behind consumer-facing applications. This security audit examines Truth Protocol's implementation to ensure it provides a secure and reliable foundation for Truth Network's prediction market ecosystem.

# Review Scope

Repository: https://github.com/Truth-Protocol-Foundation/truth-network-node/
Commit: 35d3c820b5f3e48fcdca8a30dc359caee1761a66

# Audit Findings

## F-2025-10410 Fund Lock and Market Deadlock in Trusted Markets with Unresponsive Oracle

Severity: Medium

Description
This issue affects trusted markets (those without dispute mechanisms) within the prediction markets pallet. If an oracle fails to report on a trusted market's outcome, the market becomes permanently stuck in the "Closed" state, and all user funds become irretrievably locked with no recovery mechanism.

When markets are created with dispute_mechanism: None, they are designated as trusted markets that rely solely on the designated oracle for outcome reporting. While markets with dispute mechanisms allow outsiders to report after the oracle duration expires, trusted markets explicitly prevent this fallback mechanism.

The vulnerability stems from code in report_and_resolve_market() which strictly enforces that only the designated oracle can report: pallets/prediction-markets/src/lib.rs:3453-3464:

fn report_and_resolve_market(
    origin: OriginFor<T>,
    market_id: MarketIdOf<T>,
    market_report: ReportOf<T>,
) -> DispatchResultWithPostInfo {
    <pallet_pm_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
        let sender = ensure_signed(origin.clone())?;
        // This line prevents anyone except the oracle from reporting
        ensure!(sender == market.oracle, Error::<T>::ReporterNotOracle);
        market.report = Some(market_report.clone());
        market.status = MarketStatus::Reported;
        Ok(())
    })?;
    // ...
}
The routing logic in do_report() function ensures that trusted markets always use this function: pallets/prediction-markets/src/lib.rs:2577-2586:

let weight = if market.dispute_mechanism.is_some() {
    Self::report_market_with_dispute_mechanism(
        origin,
        *market_id,
        market_report.clone(),
    )?
} else {
    Self::report_and_resolve_market(origin, *market_id, market_report.clone())?
};
In contrast, markets with dispute mechanisms use report_market_with_dispute_mechanism(), which contains logic allowing outsiders to report after the oracle duration expires.

Unfortunately, there's no administrative function to rescue a market stuck in the "Closed" state. The admin_move_market_to_resolved function exists but can only be called for markets already in "Reported" or "Disputed" states.

This creates a complete deadlock with severe consequences:

The market cannot progress beyond "Closed" state without the oracle

Users cannot redeem their shares (via redeem_shares()) as this requires the market to be "Resolved"

All funds invested in the market become permanently locked

No administrative recovery path exists

While market creators must be whitelisted, which reduces both the likelihood and severity of this vulnerability, this restriction doesn't protect against non-malicious failures such as lost private keys or death of the oracle operator. These scenarios remain realistic threats that could result in permanent loss of user funds and also contribute to blockchain storage bloat by leaving markets permanently stuck in the "Closed" state.