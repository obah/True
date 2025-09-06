import React, { ReactNode, createContext, useContext, useEffect, useState } from "react";
import { ethers } from "ethers";
import { toast } from "react-toastify";
// import { OWNERSHIP_ABI } from '../contracts/ownershipABI';
// import { AUTHENTICITY_ABI } from '../contracts/authenticityABI';
import deployedContracts from "~~/contracts/deployedContracts";

const OWNERSHIP_ADDRESS = deployedContracts[84532].TrueOwnership.address;
const AUTHENTICITY_ADDRESS = deployedContracts[84532].TrueAuthenticity.address;
const OWNERSHIP_ABI = deployedContracts[84532].TrueOwnership.abi;
const AUTHENTICITY_ABI = deployedContracts[84532].TrueAuthenticity.abi;

interface WalletContextType {
  // Connection state
  provider: ethers.BrowserProvider | null;
  signer: ethers.Signer | null;
  account: string | null;
  chainId: number;

  // Contracts
  ownershipRContract: ethers.Contract | null;
  ownershipSContract: ethers.Contract | null;
  authenticityRContract: ethers.Contract | null;
  authenticitySContract: ethers.Contract | null;

  // User data
  isUserRegistered: boolean;
  userRegisteredName: string;
  isManufacturerRegistered: boolean;
  manufacturerRegisteredName: string;

  // Functions
  connectWallet: () => Promise<void>;
  checkUserRegistration: (address?: string) => Promise<void>;
  checkManufacturerRegistration: (address?: string) => Promise<void>;
}

const WalletContext = createContext<WalletContextType | undefined>(undefined);

export const useWallet = () => {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error("useWallet must be used within a WalletProvider");
  }
  return context;
};

interface WalletProviderProps {
  children: ReactNode;
}

export const WalletProvider: React.FC<WalletProviderProps> = ({ children }) => {
  const [provider, setProvider] = useState<ethers.BrowserProvider | null>(null);
  const [signer, setSigner] = useState<ethers.Signer | null>(null);
  const [account, setAccount] = useState<string | null>(null);
  const [chainId, setChainId] = useState<number>(0);

  const [ownershipRContract, setOwnershipRContract] = useState<ethers.Contract | null>(null);
  const [ownershipSContract, setOwnershipSContract] = useState<ethers.Contract | null>(null);
  const [authenticityRContract, setAuthenticityRContract] = useState<ethers.Contract | null>(null);
  const [authenticitySContract, setAuthenticitySContract] = useState<ethers.Contract | null>(null);

  const [isUserRegistered, setIsUserRegistered] = useState(false);
  const [userRegisteredName, setUserRegisteredName] = useState("");
  const [isManufacturerRegistered, setIsManufacturerRegistered] = useState(false);
  const [manufacturerRegisteredName, setManufacturerRegisteredName] = useState("");

  useEffect(() => {
    initializeProvider();
  }, []);

  useEffect(() => {
    // Listen for account changes
    if (typeof window.ethereum !== "undefined") {
      const handleAccountsChanged = (accounts: string[]) => {
        if (accounts.length === 0) {
          // User disconnected wallet
          disconnectWallet();
          toast.info("Wallet disconnected");
        } else if (account && accounts[0] !== account) {
          // User switched accounts
          handleAccountSwitch(accounts[0]);
        }
      };

      const handleChainChanged = (chainId: string) => {
        setChainId(parseInt(chainId, 16));
        toast.info("Network changed");
      };

      window.ethereum.on("accountsChanged", handleAccountsChanged);
      window.ethereum.on("chainChanged", handleChainChanged);

      // Cleanup listeners on unmount
      return () => {
        if (window.ethereum?.removeListener) {
          window.ethereum.removeListener("accountsChanged", handleAccountsChanged);
          window.ethereum.removeListener("chainChanged", handleChainChanged);
        }
      };
    }
  }, [account]);

  const initializeProvider = async () => {
    if (typeof window.ethereum !== "undefined") {
      const web3Provider = new ethers.BrowserProvider(window.ethereum);
      setProvider(web3Provider);

      // Initialize read-only contracts
      setOwnershipRContract(new ethers.Contract(OWNERSHIP_ADDRESS, OWNERSHIP_ABI, web3Provider));
      setAuthenticityRContract(new ethers.Contract(AUTHENTICITY_ADDRESS, AUTHENTICITY_ABI, web3Provider));

      // Check if already connected
      try {
        const accounts = await window.ethereum.request({ method: "eth_accounts" });
        if (accounts.length > 0) {
          await connectExistingWallet(web3Provider, accounts[0]);
        }
      } catch (error) {
        console.error("Error checking existing connection:", error);
      }
    } else {
      toast.error("Please install MetaMask!");
    }
  };

  const connectExistingWallet = async (web3Provider: ethers.BrowserProvider, address: string) => {
    try {
      const signerInstance = await web3Provider.getSigner();
      const network = await web3Provider.getNetwork();

      setChainId(Number(network.chainId));
      setSigner(signerInstance);
      setAccount(address);

      // Initialize signed contracts
      setOwnershipSContract(new ethers.Contract(OWNERSHIP_ADDRESS, OWNERSHIP_ABI, signerInstance));
      setAuthenticitySContract(new ethers.Contract(AUTHENTICITY_ADDRESS, AUTHENTICITY_ABI, signerInstance));

      // Check registrations
      await checkUserRegistration(address);
      await checkManufacturerRegistration(address);
    } catch (error) {
      console.error("Error connecting existing wallet:", error);
    }
  };

  const handleAccountSwitch = async (newAccount: string) => {
    if (provider) {
      try {
        const signerInstance = await provider.getSigner();
        setAccount(newAccount);
        setSigner(signerInstance);

        // Update signed contracts
        setOwnershipSContract(new ethers.Contract(OWNERSHIP_ADDRESS, OWNERSHIP_ABI, signerInstance));
        setAuthenticitySContract(new ethers.Contract(AUTHENTICITY_ADDRESS, AUTHENTICITY_ABI, signerInstance));

        // Check registrations for new account
        await checkUserRegistration(newAccount);
        await checkManufacturerRegistration(newAccount);

        toast.success(`Switched to: ${newAccount.slice(0, 6)}...${newAccount.slice(-4)}`);
      } catch (error) {
        console.error("Error switching account:", error);
      }
    }
  };

  const connectWallet = async (): Promise<void> => {
    if (!provider) {
      toast.error("MetaMask not detected");
      return;
    }

    try {
      if (!account) {
        // const accounts = await window.ethereum.request({ method: "eth_requestAccounts" });
        const signerInstance = await provider.getSigner();

        const network = await provider.getNetwork();
        setChainId(Number(network.chainId));

        const address = await signerInstance.getAddress();
        setSigner(signerInstance);
        setAccount(address);

        // Initialize signed contracts
        setOwnershipSContract(new ethers.Contract(OWNERSHIP_ADDRESS, OWNERSHIP_ABI, signerInstance));
        setAuthenticitySContract(new ethers.Contract(AUTHENTICITY_ADDRESS, AUTHENTICITY_ABI, signerInstance));

        console.log("Chain ID", network.chainId);

        // Check registrations
        await checkUserRegistration(address);
        await checkManufacturerRegistration(address);

        toast.success(`Connected: ${address.slice(0, 6)}...${address.slice(-4)}`);
      } else {
        // Disconnect wallet
        disconnectWallet();
        toast.success("Wallet disconnected");
      }
    } catch (error: any) {
      toast.error(`Error: ${error.message}`);
    }
  };

  const disconnectWallet = () => {
    setSigner(null);
    setAccount(null);
    setIsUserRegistered(false);
    setUserRegisteredName("");
    setIsManufacturerRegistered(false);
    setManufacturerRegisteredName("");

    if (provider) {
      const network = provider.getNetwork();
      network.then(net => setChainId(Number(net.chainId)));

      // Reset to read-only contracts
      setOwnershipRContract(new ethers.Contract(OWNERSHIP_ADDRESS, OWNERSHIP_ABI, provider));
      setAuthenticityRContract(new ethers.Contract(AUTHENTICITY_ADDRESS, AUTHENTICITY_ABI, provider));
    }

    setOwnershipSContract(null);
    setAuthenticitySContract(null);
  };

  const checkUserRegistration = async (address?: string) => {
    const targetAddress = address || account;
    if (!ownershipRContract || !targetAddress) return;

    try {
      const user = await ownershipRContract.getUser(targetAddress);
      if (user.isRegistered && user.username && user.username.trim() !== "") {
        setIsUserRegistered(true);
        setUserRegisteredName(user.username);
      } else {
        setIsUserRegistered(false);
        setUserRegisteredName("");
      }
    } catch (error) {
      // User not registered, which is fine
      console.error(error);
      setIsUserRegistered(false);
      setUserRegisteredName("");
    }
  };

  const checkManufacturerRegistration = async (address?: string) => {
    const targetAddress = address || account;
    if (!authenticityRContract || !targetAddress) return;

    try {
      const manufacturer = await authenticityRContract.getManufacturer(targetAddress);
      if (manufacturer.name && manufacturer.name.trim() !== "") {
        setIsManufacturerRegistered(true);
        setManufacturerRegisteredName(manufacturer.name);
      } else {
        setIsManufacturerRegistered(false);
        setManufacturerRegisteredName("");
      }
    } catch (error) {
      console.error(error);
      // Manufacturer not registered, which is fine
      setIsManufacturerRegistered(false);
      setManufacturerRegisteredName("");
    }
  };

  const value: WalletContextType = {
    provider,
    signer,
    account,
    chainId,
    ownershipRContract,
    ownershipSContract,
    authenticityRContract,
    authenticitySContract,
    isUserRegistered,
    userRegisteredName,
    isManufacturerRegistered,
    manufacturerRegisteredName,
    connectWallet,
    checkUserRegistration,
    checkManufacturerRegistration,
  };

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
};
