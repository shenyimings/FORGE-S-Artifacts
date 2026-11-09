import { ProgramTestContext } from "solana-bankrun";
import {
    createPartnerMetadata,
} from "./instructions";
import { VirtualCurveProgram } from "./utils/types";
import { Keypair } from "@solana/web3.js";
import { startTest } from "./utils";
import {
    createVirtualCurveProgram,
} from "./utils";

describe("Create partner metadata", () => {
    let context: ProgramTestContext;
    let partner: Keypair;
    let user: Keypair;
    let program: VirtualCurveProgram;

    before(async () => {
        context = await startTest();
        user = context.payer;
        partner = Keypair.generate();
        program = createVirtualCurveProgram();
    });

    it("Partner create a metadata", async () => {
        await createPartnerMetadata(
            context.banksClient,
            program,
            {
                name: "Moonshot",
                website: "moonshot.com",
                logo: "https://raw.githubusercontent.com/MeteoraAg/token-metadata/main/meteora_permission_lp.png",
                feeClaimer: partner,
                payer: user,
            }
        );
    });
});
