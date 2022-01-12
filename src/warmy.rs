#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::TopEncode;

const NFT_AMOUNT: u32 = 1000;


#[elrond_wasm::contract]
pub trait WarmyDonation {


    #[init]
    fn init(&self) {
    	self.nft_left().set(&NFT_AMOUNT);
    }

    //-----------------------------------------------------//
    //                Storage Mappers		               //
    //-----------------------------------------------------//

    #[view(amountRaised)]
    #[storage_mapper("amount_raised")]
	fn amount_raised(&self) -> SingleValueMapper<BigUint>;

    #[view(NFTBought)]
    #[storage_mapper("nft_bought")]
	fn nft_bought(&self) -> SingleValueMapper<u32>;

    #[view(NFTLeft)]
    #[storage_mapper("nft_left")]
	fn nft_left(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("nftNonce")]
    fn nft_nonce_mapper(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("nftCreated")]
    fn nft_created(&self) -> SingleValueMapper<bool>;


    //-----------------------------------------------------//
    //             Endpoints (donate and claim) 		   //
    //-----------------------------------------------------//


    #[payable("EGLD")]
	#[endpoint(donate)]
	fn donate(&self, #[payment] payment: BigUint) -> SCResult<()> {

		require!(self.nft_created().get() == true, "NFT has not been created yet");

		// Min amount to get an NFT is 0.05EGLD (can be written in Uint64)
		let min_amount_for_nft = BigUint::from(50000000000000000_u64); 
		let caller = self.blockchain().get_caller();

		if (payment>=min_amount_for_nft) {

			require!(self.nft_left().get()>0, "All NFTs have been bought");

			let nft_amount = BigUint::from(1u32);
			let nonce = self.nft_nonce_mapper().get();
			let nft_id = self.nft_token_id().get();
			let thanks = "Here is your NFT. Multumesc !";


			self.send().direct(&caller, &nft_id, nonce, &nft_amount, thanks.as_bytes());

			self.nft_bought().update(|nft_bought| *nft_bought += 1);
			self.nft_left().update(|nft_left| *nft_left -= 1);
		}
		self.amount_raised().update(|amount_raised| *amount_raised += payment);

		Ok(())
	}


	#[only_owner]
	#[endpoint(claim)]
	fn claim(&self) -> SCResult<()>{

		let owner = self.blockchain().get_owner_address();
		let balance_sc = self.blockchain().get_sc_balance(&TokenIdentifier::egld(),0);

		self.send().direct_egld(&owner, &balance_sc, &[]);
		self.amount_raised().clear();


		Ok(())
	}


	#[only_owner]
	#[endpoint(claimNFT)]
	fn claim_nft_left(&self) -> SCResult<()>{

		let nft_id = self.nft_token_id().get();
		let owner = self.blockchain().get_owner_address();
		let nonce = self.nft_nonce_mapper().get();
		let nft_amount = BigUint::from(self.nft_left().get());

		self.send().direct(&owner, &nft_id, nonce, &nft_amount, &[]);
        self.nft_left().clear();


		Ok(())
	}


    //-----------------------------------------------------//
    //                NFT Creation  		               //
    //-----------------------------------------------------//

    // The functions below are inspired from an Elrond template

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
    ) -> SCResult<AsyncCall> {
        require!(self.nft_token_id().is_empty(), "Token already issued");

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .issue_semi_fungible(
                issue_cost,
                &token_name,
                &token_ticker,
                SemiFungibleTokenProperties {
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_change_owner: false,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback()))
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) -> SCResult<AsyncCall> {
        self.require_token_issued()?;


		let mut roles = Vec::new();
		roles.push(EsdtLocalRole::NftCreate);
		roles.push(EsdtLocalRole::NftAddQuantity);

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                (&roles).into_iter().cloned(),
            )
            .async_call())
    }



    #[allow(clippy::too_many_arguments)]
    fn create_nft_with_attributes<T: TopEncode>(
        &self,
        name: ManagedBuffer,
        royalties: BigUint,
        attributes: T,
        uri: ManagedBuffer,
    ) -> SCResult<u64>  {

        self.require_token_issued()?;
        self.require_local_roles_set()?;


        let nft_token_id = self.nft_token_id().get();

        let mut serialized_attributes = Vec::new();
        attributes.top_encode(&mut serialized_attributes)?;

        let attributes_hash = self.crypto().sha256(&serialized_attributes);
        let hash_buffer = ManagedBuffer::from(attributes_hash.as_bytes());

        let mut uris = ManagedVec::new();
        uris.push(uri);

        let nft_nonce = self.send().esdt_nft_create(
            &nft_token_id,
            &BigUint::from(NFT_AMOUNT),
            &name,
            &royalties,
            &hash_buffer,
            &attributes,
            &uris,
        );

        self.nft_nonce_mapper().set(&nft_nonce);
        Ok(nft_nonce)
    }

    #[allow(clippy::too_many_arguments)]
    #[only_owner]
    #[endpoint(createNft)]
    fn create_nft(
        &self,
        name: ManagedBuffer,
        uri: ManagedBuffer,
    ) -> SCResult<u64> {


        let attributes = "metadata:https://ipfs.io/ipfs/QmP5nJecZ9BbsVmXFs9hZ7ZGvL92AtqNdpnyquP2QF1ypd;tags:Beniamin,Elrond,FullMoon,Tolkien,WarmyDonation";
        let royalties = BigUint::from(0_u32); // No royalties


        self.nft_created().set(&true);
        self.create_nft_with_attributes(
            name,
            royalties,
            attributes,
            uri,)
    }



    fn require_token_issued(&self) -> SCResult<()> {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
        Ok(())
    }

    fn require_local_roles_set(&self) -> SCResult<()> {
        let nft_token_id = self.nft_token_id().get();
        let roles = self.blockchain().get_esdt_local_roles(&nft_token_id);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "NFTCreate role not set"
        );

        require!(
            roles.has_role(&EsdtLocalRole::NftAddQuantity),
            "NftAddQuantity role not set"
        );

        Ok(())
    }


    #[callback]
    fn issue_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id);
            },
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
            },
        }
    }



}
