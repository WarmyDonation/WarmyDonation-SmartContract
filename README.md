# Explanation

## The code

You will find the source code in the src directory. The smart contract alows the creation of an NFT collection which is based on an Elrond template. Additional functions are donate, claim and claimNFT. 

The function donate is callable by anyone. A user can donate any amount he wants, however if he donates at least 0.05EGLD he will receive an NFT from the previously created collection.
The function claim is callable by the owner and will be called at the end of the donation-based crowdfunding. 
The function claimNFT is useful to claim all the "unsold" NFTs from the smart contract at the end of the crowdfunding. Those will be listed on emoon at the price of 0.05EGLD.


## Deploying the smart contract

You can easily deploy the smart contract by using the bash script provided ("deploy"). It uses erdpy : 

```
erdpy contract deploy
```
Some parameters are necessary. The only file missing is the wallet's pem, you can create a pem with your own wallet.

## Creating the NFT

To create the NFT we need to first create the collection by calling the issueTokens function (the script "create_collection" gives the details).
Then we need to set the roles (add quantity and create nft) using again another bash script. Once done we can finally create the NFT with the script "create_NFT".
More info can be find in this video : https://www.youtube.com/watch?v=rGGNCeSUlRI.




