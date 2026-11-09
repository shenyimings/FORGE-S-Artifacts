# Introduction

We express our gratitude to the Civic Technologies, Inc. team for the collaborative engagement that enabled the execution of this Smart Contract Security Assessment.

Civic is a leading provider of identity and access management tools.


# Review Scope

Repository	https://github.com/identity-com/on-chain-identity-gateway
Commit	d94bfee1a35b533583efc1b2151a9224b1a4b305

# Audit Findings

## F-2023-0982 Denial Of Service State

Status
fixed
Severity
High
Description
Note: this could be a false positive - however, this was not possible to confirm from the functional requirements.

program::processor::issue_vanilla allocates the size for a new token account as equal to the size (of Borsh encoding) of the newly created GatewayToken instance.

Since GatewayToken contains several Optional-typed fields (parentgatewaytoken, owneridentity, expiretime), the encoding size of an instance may vary depending on whether some of the optionals are non-None. This is because None is encoded compactly in Borsh i.e. a field encoding is small if the value is None, and may be larger if the value is different than None.

In particular, program::processor::issue_vanilla does not set expire_time if it was not given to the method. The current space allocation approach leads to the impossibility of setting the field later.

If expire_time is not given to the method, the GatewayToken instance initial size will be 101. If later for that token program::processor::update_expiry is called, the token size will become 109, and the execution will fail to write it to the account. In other words, if a token is issued without expiration time, the expiration time cannot be set later. Note that this also blocks program::processor::expire_token for such a token, because the function works by setting an expiration time in the past.

## F-2023-0983 Improper Account Funding

Status
fixed
Severity
Medium
Description
The usage of Rent::default() may cause insufficient or excessive funding of a newly created account depending on the underlying blockchain configuration/state. In particular, insufficient funding may lead to an unexpected purging of the account.

## F-2023-0995 Confusing Code

Status
unfixed
Severity
Low
Description
The parameter name gatekeeper could mean “gatekeeper authority” or “gatekeeper account”.