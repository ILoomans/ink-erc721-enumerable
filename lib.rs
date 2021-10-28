#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
// use std::collections::HashMap;

#[ink::contract]
mod Nft {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_prelude::vec::Vec;

    use ink_prelude::string::String;
    use ink_storage::collections::{
        hashmap::Entry, HashMap as StorageHashMap, Vec as StorageVec,
    };
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use ink_storage::Pack;

    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u32;

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout,
    )]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct TokenFeatures {
        pub name: String,
        pub image: String,
        pub has_discount: bool,
        pub discount: String,
        pub issuer: AccountId,
    }

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout,
    )]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct IssuerFeatures {
        pub name: String,
        pub status: bool,
    }

    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout,
    )]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct NFTSetFeatures {
        pub name: String,
        pub description: String,
        pub status: bool,
        pub owner: AccountId,
    }

    // perhaps let anyone with these combination of tokens to start it
    #[derive(
        Debug, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout,
    )]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct SwapOffer {
        pub maker: AccountId,
        pub bids: Vec<u32>,
        pub asks: Vec<u32>,
        pub recepient: AccountId,
    }

    // nested hashmap
    #[ink(storage)]
    pub struct Erc721 {
        /// Mapping from token to owner.
        token_owner: StorageHashMap<TokenId, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: StorageHashMap<TokenId, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: StorageHashMap<AccountId, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,
        /// Token Features
        token_features: StorageHashMap<TokenId, TokenFeatures>,
        /// issuer features
        nft_issuer: StorageHashMap<AccountId, IssuerFeatures>,
        /// Owner of index implementation
        owned_tokens: StorageHashMap<(AccountId, u32), u32>,
        
        owned_tokens_index: StorageHashMap<TokenId, u32>,

        owned_nft_set: StorageHashMap<(AccountId, u32), u32>,
        owned_nft_set_index: StorageHashMap<u32, u32>,
        owned_nft_set_count: StorageHashMap<AccountId, u32>,

        owned_set_tokens: StorageHashMap<(u32, u32), u32>,
        owned_set_tokens_index: StorageHashMap<u32, u32>,
        owned_set_tokens_count: StorageHashMap<u32, u32>,

        /// Index of owned tokens
        contract_owner: AccountId,
        nft_set: StorageHashMap<u32, NFTSetFeatures>,

        swaps: StorageHashMap<u32, SwapOffer>,

        nft_bid_swap: StorageHashMap<(AccountId, u32), u32>,
        nft_bid_swap_index: StorageHashMap<u32, u32>,
        nft_bid_swap_count: StorageHashMap<AccountId, u32>,

        nft_ask_swap: StorageHashMap<(AccountId, u32), u32>,
        nft_ask_swap_index: StorageHashMap<u32, u32>,
        nft_ask_swap_count: StorageHashMap<AccountId, u32>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotRemove,
        CannotFetchValue,
        NotAllowed,
        NotIssuer,
        NotContractOwner,
        SetExists,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token Swap occurs.
    #[ink(event)]
    pub struct SwapProposal {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a swap bid occurs.
    #[ink(event)]
    pub struct SwapAccepted {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    impl Erc721 {
        /// Creates a new ERC721 token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            let contract_owner = Self::env().caller();

            Self {
                token_owner: Default::default(),
                token_approvals: Default::default(),
                owned_tokens_count: Default::default(),
                operator_approvals: Default::default(),
                token_features: Default::default(),
                owned_tokens: Default::default(),
                owned_tokens_index: Default::default(),
                nft_issuer: Default::default(),
                contract_owner,
                nft_set: Default::default(),
                owned_nft_set_count: Default::default(),
                owned_nft_set: Default::default(),
                owned_nft_set_index: Default::default(),
                owned_set_tokens: Default::default(),
                owned_set_tokens_index: Default::default(),
                owned_set_tokens_count: Default::default(),
                swaps: Default::default(),
                nft_bid_swap: Default::default(),
                nft_bid_swap_index: Default::default(),
                nft_bid_swap_count: Default::default(),
                nft_ask_swap: Default::default(),
                nft_ask_swap_index: Default::default(),
                nft_ask_swap_count: Default::default(),
            }
        }

        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u32 {
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        pub fn balance_of_bids(&self, owner: AccountId) -> u32 {
            *self.nft_bid_swap_count.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn bid_of_owner_by_index(&self, owner: AccountId, index: u32) -> u32 {
            *self.nft_bid_swap.get(&(owner, index)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn balance_of_asks(&self, owner: AccountId) -> u32 {
            *self.nft_ask_swap_count.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn ask_of_owner_by_index(&self, owner: AccountId, index: u32) -> u32 {
            *self.nft_ask_swap.get(&(owner, index)).unwrap_or(&0)
        }


        #[ink(message)]
        pub fn get_swap(
            &self,
            swapid: u32,
        ) -> (
            Option<AccountId>,
            Option<AccountId>,
            Option<Vec<u32>>,
            Option<Vec<u32>>,
        ) {
            let maker = self.swaps.get(&swapid).map(|v| v.maker.clone());
            let bids = self.swaps.get(&swapid).map(|v| v.bids.clone());

            let asks = self.swaps.get(&swapid).map(|v| v.asks.clone());
            let recepient = self.swaps.get(&swapid).map(|v| v.recepient.clone());
            return (recepient, maker, bids, asks);
        }

        // get the swaps of owner by index

        #[ink(message)]
        pub fn get_token(
            &self,
            id: TokenId,
        ) -> (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<bool>,
            Option<AccountId>,
        ) {
            let name = self.token_features.get(&id).map(|v| v.name.clone());
            let image = self.token_features.get(&id).map(|v| v.image.clone());
            let discount = self.token_features.get(&id).map(|v| v.discount.clone());
            let has_discount =
                self.token_features.get(&id).map(|v| v.has_discount.clone());
            let issuer = self.token_features.get(&id).map(|v| v.issuer.clone());
            return (name, image, discount, has_discount, issuer);
        }

        #[ink(message)]
        pub fn token_of_owner_by_index(&self, owner: AccountId, index: u32) -> u32 {
            *self.owned_tokens.get(&(owner, index)).unwrap_or(&0)
        }

        /// Returns the owner of the token.
        #[ink(message)]
        pub fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            self.token_owner.get(&id).cloned()
        }

        // The Issuer Set Balance
        #[ink(message)]
        pub fn issuer_set_balance(&self, of: AccountId) -> u32 {
            *self.owned_nft_set_count.get(&of).unwrap_or(&0)
        }

        // Enumerate Issuer Set Balance

        #[ink(message)]
        pub fn set_of_owner_by_index(&self, owner: AccountId, index: u32) -> u32 {
            *self.owned_nft_set.get(&(owner, index)).unwrap_or(&0)
        }

        // the balance of nft tokens
        #[ink(message)]
        pub fn nft_set_balance(&self, setId: u32) -> u32 {
            *self.owned_set_tokens_count.get(&setId).unwrap_or(&0)
        }

        // the balance of nft tokens
        #[ink(message)]
        pub fn nft_by_set_index(&self, setId: u32, index: u32) -> u32 {
            *self.owned_set_tokens.get(&(setId, index)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn get_nft_set(&self, id: TokenId) -> (Option<String>, Option<String>) {
            let name = self.nft_set.get(&id).map(|v| v.name.clone());
            let description = self.nft_set.get(&id).map(|v| v.description.clone());
            return (name, description);
        }

        #[ink(message)]
        pub fn issuer_name(&self, to: AccountId) -> Option<String> {
            self.nft_issuer.get(&to).map(|v| v.name.clone())
        }

        #[ink(message)]
        pub fn is_issuer(&self, to: AccountId) -> Option<bool> {
            self.nft_issuer.get(&to).map(|v| v.status.clone())
        }

        // #[ink(message)]
        // pub fn swap_status(&self, swapid: u32) -> Option<bool>{
        // self.swaps.get(&swapid).map(|v| v.status.clone())
        // }

        /// Approve issuer to mint contract.
        #[ink(message)]
        pub fn set_issuer(
            &mut self,
            to: AccountId,
            name: String,
            status: bool,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller == self.contract_owner {
                self.nft_issuer.insert(to, IssuerFeatures { name, status });
                Ok(())
            } else {
                return Err(Error::NotContractOwner);
            }
        }

        #[ink(message)]
        pub fn create_nft_set(
            &mut self,
            id: u32,
            name: String,
            description: String,
            status: bool,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let stat = self.nft_issuer.get(&caller).map(|v| v.status.clone());
            if stat == Some(true) {
                let exists = self.nft_set.get(&id).map(|v| v.status.clone());
                if exists != Some(true) {
                    self.nft_set.insert(
                        id,
                        NFTSetFeatures {
                            name,
                            description,
                            status,
                            owner: caller,
                        },
                    );
                    self.add_set_to_enumeration(caller, id);
                    Ok(())
                } else {
                    return Err(Error::NotIssuer);
                }
            } else {
                return Err(Error::NotIssuer);
            }
        }


        #[ink(message, payable)]
        pub fn mint(
            &mut self,
            setid: u32,
            id: TokenId,
            name: String,
            image: String,
            has_discount: bool,
            discount: String,
            to: AccountId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            // chage that this is the owner of the nft set they are trying to deploy to
            // add register constraint
            let status = self.nft_set.get(&setid).map(|v| v.owner.clone());
            if status == Some(caller) {
                // require that set works
                self.add_token_to_owner_enumeration(&to, id)?;
                self.add_token_to(&to, id)?;
                self.add_token_to_set_enumeration(setid, id);
                self.token_features.insert(
                    id,
                    TokenFeatures {
                        name,
                        image,
                        has_discount,
                        discount,
                        issuer: caller,
                    },
                );
                self.env().transfer(to, self.env().transferred_balance());
                self.env().emit_event(Transfer {
                    from: Some(AccountId::from([0x0; 32])),
                    to: Some(to),
                    id,
                });
            } else {
                return Err(Error::NotIssuer);
            }
            Ok(())
        }
        // Make Proposal

        // Approve the other person to make the transaction for your selected tokens
        // check that at the point in time they are the owner of those tokens
        // check that at that point in time you are the owner of those transactions
        // add to your enumerable set
        // add to their enumerable set

        #[ink(message)]
        pub fn make_swap_proposal(
            &mut self,
            swapid: TokenId,
            bidtokens: Vec<u32>,
            asktokens: Vec<u32>,
            to: AccountId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            // check that at that point in time you are the owner of those transactions
            let check_bid = self.is_owner_of_tokens(caller, bidtokens.clone());
            let check_ask = self.is_owner_of_tokens(to, asktokens.clone());
            // check that swap id doesnt exist
            if check_bid == true && check_ask == true {
                // Approve the other person to make the transaction for your selected tokens
                // add swap == true condition
                for f in bidtokens.iter() {
                    self.approve_for(&to, *f);
                }
                self.swaps.insert(
                    swapid,
                    SwapOffer {
                        maker: caller,
                        bids: bidtokens.clone(),
                        asks: asktokens.clone(),
                        recepient: to,
                    },
                );
                let ask_address_length = *self.nft_ask_swap_count.get(&to).unwrap_or(&0);
                // let length = self.balance_of(*to);
                self.nft_ask_swap.insert((to, ask_address_length), swapid);
                self.nft_ask_swap_index.insert(swapid, ask_address_length);
                let ask_entry = self.nft_ask_swap_count.entry(to);
                increase_counter_of(ask_entry);
                let bid_address_length =
                    *self.nft_bid_swap_count.get(&caller).unwrap_or(&0);
                // let length = self.balance_of(*to);
                self.nft_bid_swap
                    .insert((caller, bid_address_length), swapid);
                self.nft_bid_swap_index.insert(swapid, bid_address_length);
                let bid_entry = self.nft_bid_swap_count.entry(caller);
                increase_counter_of(bid_entry);
                self.env().emit_event(SwapProposal {
                    from: Some(caller),
                    to: Some(to),
                    id: swapid,
                });
                // increment the length by one

                // add swap id to the callers enumerableset
                // let length = self.balance_of(*to);
                // self.owned_tokens.insert((*to,length),id);
                // self.owned_tokens_index.insert(id,length);
                // incnsferement the length by one

                return Ok(());
            } else {
                return Err(Error::NotOwner);
            }
        }

        #[ink(message)]
        pub fn reject_trade(&mut self, swapid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            let maker = self
                .swaps
                .get(&swapid)
                .map(|v| v.maker.clone())
                .unwrap_or(Default::default());
            let recepient = self
                .swaps
                .get(&swapid)
                .map(|v| v.recepient.clone())
                .unwrap_or(Default::default());

            if caller == maker || caller == recepient {
                self.remove_swap_from_ask_enumeration(&recepient, swapid);
                self.remove_swap_from_bid_enumeration(&maker, swapid);
                Ok(())
            } else {
                return Err(Error::NotOwner);
            }
        }

        #[ink(message)]
        pub fn accept_trade(&mut self, swapid: u32) -> Result<(), Error> {
            // if the swap is rejected, or already accepted it is removed from this index with the take function (to be tested for 100% certainty)
            let status = self.nft_ask_swap_index.contains_key(&swapid);
            if status == true {
                let maker = self.swaps.get(&swapid).map(|v| v.maker.clone()).unwrap_or(Default::default());
                let bid_tokens = self.swaps.get(&swapid).map(|v| v.bids.clone()).unwrap_or(Default::default());
                let ask_tokens = self.swaps.get(&swapid).map(|v| v.asks.clone()).unwrap_or(Default::default());
                // let bids: Vec<_> = bid_tokens.iter().copied().collect();
                // let asks: Vec<_> = ask_tokens.iter().copied().collect();
                let caller = self.env().caller();
                let stat_1 = self.is_owner_of_tokens(maker,bid_tokens.clone());
                let stat_2 = self.is_owner_of_tokens(caller,ask_tokens.clone());
                if stat_1 == true && stat_2 == true {
                    // send the tokens of the caller
                    for t in ask_tokens.clone().iter(){
                        self.transfer_token_from(&caller,&maker,*t);
                    }

                    for t in bid_tokens.clone().iter(){
                        self.transfer_token_from(&maker,&caller,*t);
                    }
                    self.remove_swap_from_ask_enumeration(&caller, swapid);
                    self.remove_swap_from_bid_enumeration(&maker, swapid);
                    self.env().emit_event(SwapAccepted {
                        from: Some(maker),
                        to: Some(caller),
                        id: swapid,
                    });
                    Ok(())
                } else {
                    // return Err(Error::NotOwner)
                    return Err(Error::NotOwner);
                }
            // is maker the owner of
            // is the caller the owner of
            } else {
                return Err(Error::NotOwner);
            }
        }

        #[ink(message)]
        pub fn is_owner_of_tokens(&mut self, owner: AccountId, tokens: Vec<u32>) -> bool {
            for f in tokens.iter() {
                let ownerOf = self.owner_of(*f);
                if ownerOf != Some(owner) {
                    return false;
                }
            }
            return true;
        }

        // Set nft group -> we have to test that we are an issuer


        /// Returns the approved account ID for this token if any.
        // #[ink(message)]
        // pub fn get_approved(&self, id: TokenId) -> Option<AccountId> {
        // self.token_approvals.get(&id).cloned()
        // }

        /// Returns `true` if the operator is approved by the owner.
        // #[ink(message)]
        // pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
        // self.approved_for_all(owner, operator)
        // }

        /// Approves or disapproves the operator for all tokens of the caller.
        // #[ink(message)]
        // pub fn set_approval_for_all(
        // &mut self,
        // to: AccountId,
        // approved: bool,
        // ) -> Result<(), Error> {
        // self.approve_for_all(to, approved)?;
        // Ok(())
        // }

        /// Approves the account to transfer the specified token on behalf of the caller.
        // #[ink(message)]
        // pub fn approve(&mut self, to: AccountId, id: TokenId) -> Result<(), Error> {
        // self.approve_for(&to, id)?;
        // Ok(())
        // }

        /// Transfers the token from the caller to the given destination.

        #[ink(message)]
        pub fn transfer(
            &mut self,
            destination: AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, id)?;
            Ok(())
        }

        // swap

        // propose swap

        // package the tokens you want to swap -> corresponding id is given to it
        // map the tokens 0,1,2,3 and have a length field
        // has its own mapping

        // package the tokens you want to swap for -> corresponding id is given
        // map the tokens 0,1,2,3 and have a length field

        // address points to the swap pairs in an enumerable manner
        // address pounts to the swapid - 0 is incoming 1 is outgoing
        // enumerable to out offer
        // enumerable for in offer

        // Could be good place for a struct

        // Struct for incoming proposals (needs isactive)
        // accept incoming ones
        // still need an array for both

        // Struct for outgoing proposals (needs isactive)

        // swap pairs? or to just the incoming offers

        // accept swap
        // tokens get send to the user

        // multiple

        // proposal -> your tokens you want to swap .... tokens you want to gain

        // This one cannot be public for swap method
        /// Transfer approved or owned token.
        // pub fn transfer_from(
        // &mut self,
        // from: AccountId,
        // to: AccountId,
        // id: TokenId,
        // ) -> Result<(), Error> {
        // self.transfer_token_from(&from, &to, id)?;
        // Ok(())
        // }

        /// Creates a new token. Still need to add owner


        /// Deletes an existing token. Only the owner can burn the token.
        // #[ink(message)]
        // pub fn burn(&mut self, id: TokenId) -> Result<(), Error> {
        // let caller = self.env().caller();
        // let Self {
        // token_owner,
        // owned_tokens_count,
        // ..
        // } = self;
        // let occupied = match token_owner.entry(id) {
        // Entry::Vacant(_) => return Err(Error::TokenNotFound),
        // Entry::Occupied(occupied) => occupied,
        // };
        // if occupied.get() != &caller {
        // return Err(Error::NotOwner)
        // };
        // decrease_counter_of(owned_tokens_count, &caller)?;
        // occupied.remove_entry();
        // self.env().emit_event(Transfer {
        // from: Some(caller),
        // to: Some(AccountId::from([0x0; 32])),
        // id,
        // });
        // Ok(())
        // }

        // #[ink(message)]
        // pub fn last_token_index(&mut self,from: AccountId) -> u32 {
        // let balance = self.balance_of_or_zero(&from) -1;
        // return balance
        // }

        // #[ink(message)]
        // pub fn token_index(&self, owner: AccountId, id: u32) -> u32 {
        // let tokenIndex = *self.owned_tokens_index.get(&id).unwrap_or(&0);
        // return tokenIndex
        // }

        // #[ink(message)]
        // pub fn last_token_id(&self, from: AccountId, lasttokenindex: u32) -> u32 {
        // let lastTokenId = self.owned_tokens.get_mut(&(from, lasttokenindex))..unwrap_or(&0);
        // return lastTokenId
        // }

        // need nft set balance

        // need setbyindex function

        pub fn add_token_to_set_enumeration(
            &mut self,
            setId: u32,
            tokenid: TokenId,
        ) -> Result<(), Error> {
            //add to token count bit
            let length = *self
                .owned_set_tokens_count
                .get_mut(&setId)
                .unwrap_or(&mut 0);
            // Adds token to max length
            self.owned_set_tokens.insert((setId, length), tokenid);
            // //
            self.owned_set_tokens_index.insert(tokenid, length);
            /// first add the token then increment the count
            self.increment_nft_token_set(setId);
            // Ok(())
            Ok(())
        }

        pub fn increment_nft_token_set(&mut self, id: u32) -> Result<(), Error> {
            let entry = *self.owned_set_tokens_count.get_mut(&id).unwrap_or(&mut 0);
            let new = entry + 1;
            self.owned_set_tokens_count.insert(id, new);
            Ok(())
        }

        // mot sure this is what we want
        // could enumerate between the address of the owner and their sets

        pub fn add_set_to_enumeration(
            &mut self,
            to: AccountId,
            setid: u32,
        ) -> Result<(), Error> {
            //add to token count bit
            let length = *self.owned_nft_set_count.get_mut(&to).unwrap_or(&mut 0);
            // Adds token to max length
            self.owned_nft_set.insert((to, length), setid);
            // //
            self.owned_nft_set_index.insert(setid, length);
            /// first add the token then increment the count
            let entry = self.owned_nft_set_count.entry(to);

            increase_counter_of(entry);
            // Ok(())
            Ok(())
        }

        // Decrement nft enumeration

        // pub fn increment_nft_set(&mut self, id: u32) -> Result<(), Error> {
        // let entry = *self.owned_nft_set_count.get_mut(&id).unwrap_or(&mut 0);
        // let new = entry+1;
        // self.owned_nft_set_count.insert(id,new);
        // Ok(())
        // }

        pub fn add_token_to_owner_enumeration(
            &mut self,
            to: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let length = self.balance_of(*to);
            // Adds token to max length
            self.owned_tokens.insert((*to, length), id);
            //
            self.owned_tokens_index.insert(id, length);
            Ok(())
        }

        //urgent
        //maybe better to switch this to 100
        pub fn remove_swap_from_ask_enumeration(
            &mut self,
            from: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            //max token

            // nft_ask_swap: Default::default(),
            // nft_ask_swap_index: Default::default(),
            // nft_ask_swap_count: Default::default(),

            // We have 2 as an answer
            let lastTokenIndex = self.balance_of_asks(*from) - 1;
            // we have 1 as the answer
            let tokenIndex = self.nft_ask_swap_index.get(&id).unwrap_or(&0);

            //somehow test that this works
            // actual token index
            // When the token to delete is the last token, the swap operation is unnecessary
            // if statement isnt working but isnt necessary test
            if tokenIndex != &lastTokenIndex {
                //tyy changing this again to &mut
                let lastTokenId = *self
                    .nft_ask_swap
                    .get_mut(&(*from, lastTokenIndex))
                    .ok_or(Error::CannotFetchValue)?;
                self.nft_ask_swap.insert((*from, *tokenIndex), lastTokenId);
                self.nft_ask_swap_index.insert(lastTokenId, *tokenIndex);
            }

            self.nft_ask_swap_index.take(&id);

            self.nft_ask_swap.take(&(*from, lastTokenIndex));
            let Self {
                nft_ask_swap_count, ..
            } = self;
            decrease_counter_of(nft_ask_swap_count, from)?;

            Ok(())
        }

        pub fn remove_swap_from_bid_enumeration(
            &mut self,
            from: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            //max token

            // nft_bid_swap: Default::default(),
            // nft_bid_swap_index: Default::default(),
            // nft_bid_swap_count: Default::default(),

            // We have 2 as an answer
            let lastTokenIndex = self.balance_of_bids(*from) - 1;
            // we have 1 as the answer
            let tokenIndex = self.nft_bid_swap_index.get(&id).unwrap_or(&0);

            //somehow test that this works
            // actual token index
            // When the token to delete is the last token, the swap operation is unnecessary
            // if statement isnt working but isnt necessary test
            if tokenIndex != &lastTokenIndex {
                //tyy changing this again to &mut
                let lastTokenId = *self
                    .nft_bid_swap
                    .get_mut(&(*from, lastTokenIndex))
                    .ok_or(Error::CannotFetchValue)?;
                self.nft_bid_swap.insert((*from, *tokenIndex), lastTokenId);
                self.nft_bid_swap_index.insert(lastTokenId, *tokenIndex);
            }

            self.nft_bid_swap_index.take(&id);

            self.nft_bid_swap.take(&(*from, lastTokenIndex));
            let Self {
                nft_bid_swap_count, ..
            } = self;
            decrease_counter_of(nft_bid_swap_count, from)?;

            Ok(())
        }

        pub fn remove_token_from_owner_enumeration(
            &mut self,
            from: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            //max token

            // We have 2 as an answer
            let lastTokenIndex = self.balance_of_or_zero(from) - 1;
            // we have 1 as the answer
            let tokenIndex = self.owned_tokens_index.get(&id).unwrap_or(&0);

            //somehow test that this works
            // actual token index
            // When the token to delete is the last token, the swap operation is unnecessary
            // if statement isnt working but isnt necessary test
            if tokenIndex != &lastTokenIndex {
                //tyy changing this again to &mut
                let lastTokenId = *self
                    .owned_tokens
                    .get_mut(&(*from, lastTokenIndex))
                    .ok_or(Error::CannotFetchValue)?;

                //let lastTokenId = *self.owned_tokens.get(&(*from, lastTokenIndex)).unwrap_or(&0);
                //let lastTokenId = self.token_of_owner_by_index(*from,lastTokenIndex);
                //replace the token name
                self.owned_tokens.insert((*from, *tokenIndex), lastTokenId);
                self.owned_tokens_index.insert(lastTokenId, *tokenIndex);

                //.owned_tokens_index.insert(&lastTokedId,tokenIndex);

                // let status = self
                // .operator_approvals
                // .get_mut(&(caller, to))
                // .ok_or(Error::CannotFetchValue)?;
            }

            self.owned_tokens_index.take(&id);

            self.owned_tokens.take(&(*from, lastTokenIndex));
            Ok(())
        }

        /// Add token to enumeration list
        // #[ink(message)]
        // fn add_token_to_owner_enumeration(&mut self,to:AccountId,id:TokenId) -> Result<(),Error> {
        // let length = self.balance_of(to);
        // self.owned_tokens.insert((to,length),id);
        // Ok(())
        // // match self.operator_approvals.insert((caller, to), approved) {
        // // Some(_) => Err(Error::CannotInsert),
        // // None => Ok(()),
        // // }
        // }

        /// Transfers token `id` `from` the sender to the `to` AccountId.
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            };
            if !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved);
            };
            self.clear_approval(id)?;
            self.remove_token_from_owner_enumeration(from,id)?;
            self.remove_token_from(from, id)?;
            self.add_token_to_owner_enumeration(to,id)?;
            self.add_token_to(to, id)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(
            &mut self,
            from: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;
            let occupied = match token_owner.entry(id) {
                Entry::Vacant(_) => return Err(Error::TokenNotFound),
                Entry::Occupied(occupied) => occupied,
            };
            decrease_counter_of(owned_tokens_count, from)?;
            occupied.remove_entry();
            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        // add to enumeration
        fn add_token_to(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;
            let vacant_token_owner = match token_owner.entry(id) {
                Entry::Vacant(vacant) => vacant,
                Entry::Occupied(_) => return Err(Error::TokenExists),
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };
            let entry = owned_tokens_count.entry(*to);
            increase_counter_of(entry);
            vacant_token_owner.insert(*to);
            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        // fn approve_for_all(
        // &mut self,
        // to: AccountId,
        // approved: bool,
        // ) -> Result<(), Error> {
        // let caller = self.env().caller();
        // if to == caller {
        // return Err(Error::NotAllowed)
        // }
        // self.env().emit_event(ApprovalForAll {
        // owner: caller,
        // operator: to,
        // approved,
        // });
        // if self.approved_for_all(caller, to) {
        // let status = self
        // .operator_approvals
        // .get_mut(&(caller, to))
        // .ok_or(Error::CannotFetchValue)?;
        // *status = approved;
        // Ok(())
        // } else {
        // match self.operator_approvals.insert((caller, to), approved) {
        // Some(_) => Err(Error::CannotInsert),
        // None => Ok(()),
        // }
        // }
        // }

        /// Approve the passed AccountId to transfer the specified token on behalf of the message's sender.
        fn approve_for(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(id);
            if !(owner == Some(caller)
                || self.approved_for_all(owner.expect("Error with AccountId"), caller))
            {
                return Err(Error::NotAllowed);
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };

            if self.token_approvals.insert(id, *to).is_some() {
                return Err(Error::CannotInsert);
            };
            // self.env().emit_event(Approval {
            // from: caller,
            // to: *to,
            // id,
            // });
            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: TokenId) -> Result<(), Error> {
            if !self.token_approvals.contains_key(&id) {
                return Ok(());
            };
            match self.token_approvals.take(&id) {
                Some(_res) => Ok(()),
                None => Err(Error::CannotRemove),
            }
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &AccountId) -> u32 {
            *self.owned_tokens_count.get(of).unwrap_or(&0)
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            *self
                .operator_approvals
                .get(&(owner, operator))
                .unwrap_or(&false)
        }

        /// Returns true if the AccountId `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Option<AccountId>, id: TokenId) -> bool {
            let owner = self.owner_of(id);
            from != Some(AccountId::from([0x0; 32]))
                && (from == owner
                    || from == self.token_approvals.get(&id).cloned()
                    || self.approved_for_all(
                        owner.expect("Error with AccountId"),
                        from.expect("Error with AccountId"),
                    ))
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: TokenId) -> bool {
            self.token_owner.get(&id).is_some() && self.token_owner.contains_key(&id)
        }
    }

    // Repear this for

    fn decrease_counter_of(
        hmap: &mut StorageHashMap<AccountId, u32>,
        of: &AccountId,
    ) -> Result<(), Error> {
        let count = (*hmap).get_mut(of).ok_or(Error::CannotFetchValue)?;
        *count -= 1;
        Ok(())
    }

    /// Increase token counter from the `of` AccountId.

    /// Increase token counter from the `of` AccountId.
    fn increase_counter_of(entry: Entry<AccountId, u32>) {
        entry.and_modify(|v| *v += 1).or_insert(1);
    }
}

// /// Unit tests
// #[cfg(test)]
// mod tests {
// /// Imports all the definitions from the outer scope so we can use them here.
// use super::*;
// use ink_env::{
// call,
// test,
// };
// use ink_lang as ink;

// #[ink::test]
// fn mint_works() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Token 1 does not exists.
// assert_eq!(erc721.owner_of(1), None);
// // Alice does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.alice), 0);
// // Create token Id 1.
// assert_eq!(erc721.mint(1), Ok(()));
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// }

// #[ink::test]
// fn mint_existing_should_fail() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1.
// assert_eq!(erc721.mint(1), Ok(()));
// // The first Transfer event takes place
// assert_eq!(1, ink_env::test::recorded_events().count());
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Alice owns token Id 1.
// assert_eq!(erc721.owner_of(1), Some(accounts.alice));
// // Cannot create token Id if it exists.
// // Bob cannot own token Id 1.
// assert_eq!(erc721.mint(1), Err(Error::TokenExists));
// }

// #[ink::test]
// fn transfer_works() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1 for Alice
// assert_eq!(erc721.mint(1), Ok(()));
// // Alice owns token 1
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Bob does not owns any token
// assert_eq!(erc721.balance_of(accounts.bob), 0);
// // The first Transfer event takes place
// assert_eq!(1, ink_env::test::recorded_events().count());
// // Alice transfers token 1 to Bob
// assert_eq!(erc721.transfer(accounts.bob, 1), Ok(()));
// // The second Transfer event takes place
// assert_eq!(2, ink_env::test::recorded_events().count());
// // Bob owns token 1
// assert_eq!(erc721.balance_of(accounts.bob), 1);
// }

// #[ink::test]
// fn invalid_transfer_should_fail() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Transfer token fails if it does not exists.
// assert_eq!(erc721.transfer(accounts.bob, 2), Err(Error::TokenNotFound));
// // Token Id 2 does not exists.
// assert_eq!(erc721.owner_of(2), None);
// // Create token Id 2.
// assert_eq!(erc721.mint(2), Ok(()));
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Token Id 2 is owned by Alice.
// assert_eq!(erc721.owner_of(2), Some(accounts.alice));
// // Get contract address
// let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
// .unwrap_or([0x0; 32].into());
// // Create call
// let mut data =
// ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
// data.push_arg(&accounts.bob);
// // Push the new execution context to set Bob as caller
// ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
// accounts.bob,
// callee,
// 1000000,
// 1000000,
// data,
// );
// // Bob cannot transfer not owned tokens.
// assert_eq!(erc721.transfer(accounts.eve, 2), Err(Error::NotApproved));
// }

// #[ink::test]
// fn approved_transfer_works() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1.
// assert_eq!(erc721.mint(1), Ok(()));
// // Token Id 1 is owned by Alice.
// assert_eq!(erc721.owner_of(1), Some(accounts.alice));
// // Approve token Id 1 transfer for Bob on behalf of Alice.
// assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
// // Get contract address.
// let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
// .unwrap_or([0x0; 32].into());
// // Create call
// let mut data =
// ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
// data.push_arg(&accounts.bob);
// // Push the new execution context to set Bob as caller
// ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
// accounts.bob,
// callee,
// 1000000,
// 1000000,
// data,
// );
// // Bob transfers token Id 1 from Alice to Eve.
// assert_eq!(
// erc721.transfer_from(accounts.alice, accounts.eve, 1),
// Ok(())
// );
// // TokenId 3 is owned by Eve.
// assert_eq!(erc721.owner_of(1), Some(accounts.eve));
// // Alice does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.alice), 0);
// // Bob does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.bob), 0);
// // Eve owns 1 token.
// assert_eq!(erc721.balance_of(accounts.eve), 1);
// }

// #[ink::test]
// fn approved_for_all_works() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1.
// assert_eq!(erc721.mint(1), Ok(()));
// // Create token Id 2.
// assert_eq!(erc721.mint(2), Ok(()));
// // Alice owns 2 tokens.
// assert_eq!(erc721.balance_of(accounts.alice), 2);
// // Approve token Id 1 transfer for Bob on behalf of Alice.
// assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
// // Bob is an approved operator for Alice
// assert_eq!(
// erc721.is_approved_for_all(accounts.alice, accounts.bob),
// true
// );
// // Get contract address.
// let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
// .unwrap_or([0x0; 32].into());
// // Create call
// let mut data =
// ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
// data.push_arg(&accounts.bob);
// // Push the new execution context to set Bob as caller
// ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
// accounts.bob,
// callee,
// 1000000,
// 1000000,
// data,
// );
// // Bob transfers token Id 1 from Alice to Eve.
// assert_eq!(
// erc721.transfer_from(accounts.alice, accounts.eve, 1),
// Ok(())
// );
// // TokenId 1 is owned by Eve.
// assert_eq!(erc721.owner_of(1), Some(accounts.eve));
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Bob transfers token Id 2 from Alice to Eve.
// assert_eq!(
// erc721.transfer_from(accounts.alice, accounts.eve, 2),
// Ok(())
// );
// // Bob does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.bob), 0);
// // Eve owns 2 tokens.
// assert_eq!(erc721.balance_of(accounts.eve), 2);
// // Get back to the parent execution context.
// ink_env::test::pop_execution_context();
// // Remove operator approval for Bob on behalf of Alice.
// assert_eq!(erc721.set_approval_for_all(accounts.bob, false), Ok(()));
// // Bob is not an approved operator for Alice.
// assert_eq!(
// erc721.is_approved_for_all(accounts.alice, accounts.bob),
// false
// );
// }

// #[ink::test]
// fn not_approved_transfer_should_fail() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1.
// assert_eq!(erc721.mint(1), Ok(()));
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Bob does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.bob), 0);
// // Eve does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.eve), 0);
// // Get contract address.
// let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
// .unwrap_or([0x0; 32].into());
// // Create call
// let mut data =
// ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
// data.push_arg(&accounts.bob);
// // Push the new execution context to set Eve as caller
// ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
// accounts.eve,
// callee,
// 1000000,
// 1000000,
// data,
// );
// // Eve is not an approved operator by Alice.
// assert_eq!(
// erc721.transfer_from(accounts.alice, accounts.frank, 1),
// Err(Error::NotApproved)
// );
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Bob does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.bob), 0);
// // Eve does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.eve), 0);
// }

// #[ink::test]
// fn burn_works() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1 for Alice
// assert_eq!(erc721.mint(1), Ok(()));
// // Alice owns 1 token.
// assert_eq!(erc721.balance_of(accounts.alice), 1);
// // Alice owns token Id 1.
// assert_eq!(erc721.owner_of(1), Some(accounts.alice));
// // Destroy token Id 1.
// assert_eq!(erc721.burn(1), Ok(()));
// // Alice does not owns tokens.
// assert_eq!(erc721.balance_of(accounts.alice), 0);
// // Token Id 1 does not exists
// assert_eq!(erc721.owner_of(1), None);
// }

// #[ink::test]
// fn burn_fails_token_not_found() {
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Try burning a non existent token
// assert_eq!(erc721.burn(1), Err(Error::TokenNotFound));
// }

// #[ink::test]
// fn burn_fails_not_owner() {
// let accounts =
// ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
// .expect("Cannot get accounts");
// // Create a new contract instance.
// let mut erc721 = Erc721::new();
// // Create token Id 1 for Alice
// assert_eq!(erc721.mint(1), Ok(()));
// // Try burning this token with a different account
// set_sender(accounts.eve);
// assert_eq!(erc721.burn(1), Err(Error::NotOwner));
// }

// fn set_sender(sender: AccountId) {
// let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
// .unwrap_or([0x0; 32].into());
// test::push_execution_context::<Environment>(
// sender,
// callee,
// 1000000,
// 1000000,
// test::CallData::new(call::Selector::new([0x00; 4])), // dummy
// );
// }
// }
