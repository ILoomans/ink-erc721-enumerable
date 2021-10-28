
# Ink! NFT Smart Contract with Enumeration & Swaps

This is an ink! smart contract based off of the ERC721 standard. Enumeration has been added to record the ownership of tokens, together with the implementation of a token swap mechanism.
Because this contract is an extension of the ERC721 contract, documentation for now will only cover the functionality built on top of it.

Approval & ApprovalForAll Events have been taken out and replaced with SwapProposal & SwapAccepted Events. This is because the only case in which an address is approved to transfer tokens is
for the purpose of swapping tokens.
## To do

- [ ] Add Query Documentation
- [ ] Add Event Documentation

## Transactions

### set_issuer

#### Description
This allows the owner of the smart contract to set an issuer of NFT's.

#### Parameters

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `to` | `address` | Address being added to issue nft's|
| `name` | `string` | Name of the user that issues nft's|
| `status` | `bool` | Activity status of the issuer |

#### Constraints

Only the owner of the contract can sign this transaction.

### create_nft_set
#### Description
This allows a registered issuer to create an nft set, minted tokens have to belong to an nft set owned by the issuer
#### Parameters

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `id` | `int` | The id of the set being created |
| `name` | `string` | Name of the nft set being created |
| `description` | `string` | Description of the nft set being created |
| `status` | `bool` | The status of the nft set |

#### Constraints

Only a registered issuer of the contract can sign this transaction.

Only a nft set can be created if the id has not been taken yet.


### mint
#### Description
This allows a registered issuer to mint a new token and associate it to a new nft set.
#### Parameters

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `id` | `int` | The id of the set being created |
| `name` | `string` | Name of the nft set being created |
| `description` | `string` | Description of the nft set being created |
| `status` | `bool` | The status of the nft set |

#### Constraints

Only a registered issuer of the contract can sign this transaction.

Only a nft set can be created if the id has not been taken yet.

### make_swap_proposal

#### Description
Allows a user to propose a swap of tokens.

#### Parameters

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `swapid` | `int` | The id of the swap proposal being created |
| `bidtokens` | `Array<Int>` | The tokens the maker is proposing to trade |
| `asktokens` | `Array<Int>` | The tokens the maker is proposing to receive |
| `to` | `address` | The person to which the the swap proposal is being made |

#### Constraints

The signer of the transaction must own all the bid tokens.

The person to which the trade is being proposed must own all the asktokens.

### reject_trade

#### Description
Allows the maker or receiver of a swap proposal to reject the proposal.

#### Parameters

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `swapid` | `int` | The id of the swap proposal being created |


#### Constraints

Only the maker or receiver of a swap proposal can sign this transaction

### accept_trade

#### Description
Allows the maker or receiver of a swap proposal to accept the proposal.

#### Parameters

| Parameter | Type     | Description                |
| :-------- | :------- | :------------------------- |
| `swapid` | `int` | The id of the swap proposal being created |


#### Constraints

Only the maker or receiver of a swap proposal can sign this transaction


## Authors

- [@ignaceloomans](https://www.github.com/iloomans)

