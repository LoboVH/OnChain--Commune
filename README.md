# OnChain--Commune with Anchor-SOLANA
This repo contains code for running and testing 'commune' project.

## Getting Started
Use `git clone https://github.com/LoboVH/OnChain--Commune.git`to get the files in this repository onto your local machine.

Run `npm install` to get all the dependencies

## To Run locally
1. If you're running solana for the first time, generate a wallet
 ```solana
      solana-keygen new
  ```

2. You can use anchor CLI to build and emit an IDL, from which clients can be generated
  ```anchor
      anchor build
   ```

Once run, you should see your build artifacts IDL, as usual, in your target/directory

3.Deploy the program by running 
  ```anchor 
      anchor deploy
  ```
  
  Update Program-Id in lib.rs and Anchor.toml
  
4. Finally run the test, make sure to kill any instance of local validator started earlier
  ```anchor
      anchor test
  ```
 
 
