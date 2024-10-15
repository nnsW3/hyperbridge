// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.17;

import "forge-std/Script.sol";
import {ERC6160Ext20} from "@polytope-labs/erc6160/tokens/ERC6160Ext20.sol";
import {IERC6160Ext20} from "@polytope-labs/erc6160/interfaces/IERC6160Ext20.sol";
import {TokenGateway, Asset, TokenGatewayParamsExt, TokenGatewayParams, AssetMetadata} from "../src/modules/TokenGateway.sol";
import {TokenFaucet} from "../src/modules/TokenFaucet.sol";
import {CrossChainInscription} from "../src/modules/Inscriptions.sol";
import {BaseScript} from "./BaseScript.sol";
import {IIsmpHost} from "@polytope-labs/ismp-solidity/IIsmpHost.sol";

contract DeployScript is BaseScript {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER ROLE");

    function run() external  {
        // todo:
        address callDispatcher = address(1);

        if (equal(host, "sepolia") || equal(host, "ethereum")) {
            vm.startBroadcast(uint256(privateKey));
            deployInscription(SEPOLIA_HOST, admin);
            deployGateway(SEPOLIA_HOST, admin, callDispatcher);
        } else if (equal(host, "arbitrum-sepolia")) {
            vm.startBroadcast(uint256(privateKey));
            deployInscription(ARB_SEPOLIA_HOST, admin);
            deployGateway(ARB_SEPOLIA_HOST, admin, callDispatcher);
        } else if (equal(host, "optimism-sepolia")) {
            vm.startBroadcast(uint256(privateKey));
            deployInscription(OP_SEPOLIA_HOST, admin);
            deployGateway(OP_SEPOLIA_HOST, admin, callDispatcher);
        } else if (equal(host, "base-sepolia")) {
            vm.startBroadcast(uint256(privateKey));
            deployInscription(BASE_SEPOLIA_HOST, admin);
            deployGateway(BASE_SEPOLIA_HOST, admin, callDispatcher);
        } else if (equal(host, "bsc-testnet")) {
            vm.startBroadcast(uint256(privateKey));
            deployInscription(BSC_TESTNET_HOST, admin);
            deployGateway(BSC_TESTNET_HOST, admin, callDispatcher);
        } else if (equal(host, "chiado")) {
            vm.startBroadcast(uint256(privateKey));
            deployInscription(CHIADO_HOST, admin);
            deployGateway(CHIADO_HOST, admin, callDispatcher);
        }
    }

    function deployInscription(address host, address admin) public {
        CrossChainInscription c = new CrossChainInscription{salt: salt}(admin);
        c.setHost(host);
    }

    function deployGateway(address host, address admin, address callDispatcher) public {
        IERC6160Ext20 feeToken = IERC6160Ext20(IIsmpHost(host).feeToken());

        TokenGateway gateway = new TokenGateway{salt: salt}(admin);
        feeToken.grantRole(MINTER_ROLE, address(gateway));
        feeToken.grantRole(BURNER_ROLE, address(gateway));

        AssetMetadata[] memory assets = new AssetMetadata[](1);
        assets[0] = AssetMetadata({
            erc20: address(0),
            erc6160: address(feeToken),
            name: "Hyperbridge USD",
            symbol: "USDH",
            beneficiary: address(0),
            initialSupply: 0
        });

        gateway.init(
            TokenGatewayParamsExt({
                params: TokenGatewayParams({host: host, dispatcher: callDispatcher}),
                assets: assets
            })
        );
    }
}
