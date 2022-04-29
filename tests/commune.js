const anchor = require('@project-serum/anchor');
const { LAMPORTS_PER_SOL } = require('@solana/web3.js');
const { assert } = require('chai');
const { SystemProgram } = anchor.web3;

describe("commune", () => {
  console.log("Starting test....")

  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Commune;

  

  const DAY_IN_UNIX = 24 * 60 *60 * 1000;

  async function getNumberBuffer(number) {
    const totalItemAccoountBuf = Buffer.alloc(8);
    totalItemAccoountBuf.writeUIntLE(number, 0, 6);
    return totalItemAccoountBuf;
  };

  const seller = anchor.web3.Keypair.generate();

  const buyer = anchor.web3.Keypair.generate();

before(async () => {
  const signature = await program.provider.connection.requestAirdrop(
    seller.publicKey,
    1 * LAMPORTS_PER_SOL,
  );
  await program.provider.connection.confirmTransaction(signature);

  const signature2 = await program.provider.connection.requestAirdrop(
    buyer.publicKey,
    20 * LAMPORTS_PER_SOL,
  );
  await program.provider.connection.confirmTransaction(signature2);
});

  it("test inititalize market function", async () => {
  const [communeAccountPublicKey, accountBump] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("commune")],
        anchor.workspace.Commune.programId
      );

    const tx = await program.rpc.initializeMarket(
      new anchor.BN(accountBump),
      {
      accounts: {
        commune: communeAccountPublicKey,
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      },
      });

    console.log("Your transaction signature", tx);

    let communeAccount = await program.account.commune.fetch(communeAccountPublicKey);
    assert.equal(communeAccount.itemCount, 0);
    assert.equal(communeAccount.fee, 0.01 * LAMPORTS_PER_SOL);
    assert.equal(communeAccount.tax, 3);
    assert.equal(communeAccount.totalProposalCount, 0);

  });

  it("tests JoinCommune function", async () => {
    const [communeAccountPublicKey] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("commune")],
        anchor.workspace.Commune.programId
      );

    let balanceBuyer = await provider.connection.getBalance(seller.publicKey);

    const [approverAccountPublicKey, approverAccountBump] = 
    await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("approver_account"), seller.publicKey.toBuffer()],
      anchor.workspace.Commune.programId,
    );

    await program.rpc.joinCommune(
      new anchor.BN(approverAccountBump),
      {
        accounts: {
          commune: communeAccountPublicKey,
          approver: approverAccountPublicKey,
          member: seller.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [seller],
      }
    );

    let approverAccount = await program.account.approver.fetch(approverAccountPublicKey);

    assert.equal(approverAccount.approval, true);

  });

  it("tests create item function", async () => {
    const [approverAccountPublicKey] = 
    await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("approver_account"), seller.publicKey.toBuffer()],
      anchor.workspace.Commune.programId,
    );

    const [communeAccountPublicKey] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("commune")],
        anchor.workspace.Commune.programId
      );
    
    let communeAccount = await program.account.commune.fetch(communeAccountPublicKey);
  

    const itemId = await getNumberBuffer(communeAccount.itemCount.toNumber());

    const [itemAccountPublicKey, itemAccountBump] = 
        await anchor.web3.PublicKey.findProgramAddress(
          [Buffer.from("item_account"), itemId ],
          anchor.workspace.Commune.programId,
        );

    await program.rpc.createItem(
      new anchor.BN(itemAccountBump),
      communeAccount.itemCount,
      "Test Title",
      new anchor.BN(10),
      "Test Description",
      {
        accounts: {
          commune: communeAccountPublicKey,
          item: itemAccountPublicKey,
          approver: approverAccountPublicKey,
          seller: seller.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [seller],
      });
    
    let itemAccount = await program.account.item.fetch(itemAccountPublicKey);
    
    assert.equal(itemAccount.title.toString(), "Test Title");
    assert.equal(itemAccount.price.toString(), 10300000000);
    assert.equal(itemAccount.description.toString(), "Test Description");

  });

  it("tests create market sale function", async () => {
    const [approverAccountPublicKey, approverAccountBump] = 
    await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("approver_account"), buyer.publicKey.toBuffer()],
      anchor.workspace.Commune.programId,
    );

    const [communeAccountPublicKey] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("commune")],
        anchor.workspace.Commune.programId
      );

      await program.rpc.joinCommune(
      new anchor.BN(approverAccountBump),
      {
        accounts: {
          commune: communeAccountPublicKey,
          approver: approverAccountPublicKey,
          member: buyer.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [buyer],
      }
    );

    const itemId = await getNumberBuffer(0);

    const [itemAccountPublicKey] = 
        await anchor.web3.PublicKey.findProgramAddress(
          [Buffer.from("item_account"), itemId ],
          anchor.workspace.Commune.programId,
        );
    
    

    await program.rpc.createMarketSale(
    new anchor.BN(0),
      {
        accounts: {
          commune:communeAccountPublicKey,
          item: itemAccountPublicKey,
          approver: approverAccountPublicKey,
          buyer: buyer.publicKey,
          to: seller.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [buyer],
      }
    );
    let itemAccount = await program.account.item.fetch(itemAccountPublicKey);
    
    
    assert.equal(itemAccount.buyer.toString(), buyer.publicKey);
    assert.equal(itemAccount.sold, true);
  });

  it("tests add proposal function", async () => {
    const [communeAccountPublicKey] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("commune")],
        anchor.workspace.Commune.programId
      );

    const [approverAccountPublicKey] = 
    await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("approver_account"), buyer.publicKey.toBuffer()],
      anchor.workspace.Commune.programId,
    );

    let communeAccount = await program.account.commune.fetch(communeAccountPublicKey);

    const proposalId = await getNumberBuffer(communeAccount.totalProposalCount.toNumber());
  
      const [proposalAccountPublicKey, proposalAccountBump] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("proposal_account"), proposalId],
        anchor.workspace.Commune.programId,
      );

      await program.rpc.addProposal(
        new anchor.BN(proposalAccountBump),
        communeAccount.totalProposalCount,
        "test proposal title",
        "test proposal description",
        new anchor.BN(3),
        new anchor.BN(+new Date() + 7 * DAY_IN_UNIX
        ),
        {
          accounts: {
            commune: communeAccountPublicKey,
            proposal: proposalAccountPublicKey,
            approver: approverAccountPublicKey,
            user: buyer.publicKey,
            systemProgram: SystemProgram.programId,
          },
          signers: [buyer],
        }
      );

      let proposalAccount = await program.account.proposal.fetch(proposalAccountPublicKey);

      assert.equal(proposalAccount.title, "test proposal title");
      assert.equal(proposalAccount.owner.toString(), buyer.publicKey);
      assert.equal(proposalAccount.description, "test proposal description");
      console.log("proposal created at", proposalAccount.createdAt);
  });

  it("tests vote for proposal function", async () => {
    const proposalId = await getNumberBuffer(0);

    const [proposalAccountPublicKey] = 
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("proposal_account"), proposalId],
        anchor.workspace.Commune.programId,
      );

    const [voteAccountPublicKey, voteAccountBump] =
        await anchor.web3.PublicKey.findProgramAddress(
          [Buffer.from("vote_account"), proposalId, buyer.publicKey.toBuffer()],
          anchor.workspace.Commune.programId,
        );

    const [approverAccountPublicKey] = 
    await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("approver_account"), buyer.publicKey.toBuffer()],
      anchor.workspace.Commune.programId,
    );

    await program.rpc.voteForProposal(
      new anchor.BN(voteAccountBump),
      new anchor.BN(0),
      true,
      {
        accounts: {
          proposal: proposalAccountPublicKey,
          vote: voteAccountPublicKey,
          approver: approverAccountPublicKey,
          user: buyer.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [buyer],
      }
    );

    let proposalAccount = await program.account.proposal.fetch(proposalAccountPublicKey);

    assert.equal(proposalAccount.voteYes, 1);

  });

});

