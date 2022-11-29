# Elrond City Router SC

Elrond City router

## Setup

These endpoint must be called just after deploying the contract in order:

## Endpoints

### Public
- **distribute**()
Payable endpoint to be called by the ecity-minter (sending the newly minted tokens to it).  
The endpoint will distribute the received tokens according to the distribution parameters.

### Private

- **addDistribution**(address: `ManagedAddress`, percentage: `u64`)
- **removeDistribution**(address: `ManagedAddress`)
- **addToken**(token_id: `TokenIdentifier`)

## Views

### Backend

### Frontend

## DTOs

