"use client";

import React, { Suspense, useCallback, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import { parseError } from "../../utils/constants/blockchain";
import {
  TransferCodeData,
  createNotification,
  getActiveTransferCodes,
  revokeTransferCode,
} from "../../utils/lib/supabase";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import { ethers } from "ethers";
import {
  ArrowRightLeft,
  CheckCircle,
  ChevronDown,
  Key,
  Package,
  Plus,
  Scan,
  Search,
  Shield,
  User,
  X,
} from "lucide-react";
import { toast } from "react-toastify";
import { useAccount } from "wagmi";
import { useScaffoldReadContract, useScaffoldWriteContract } from "~~/hooks/scaffold-eth";

const UserDashboardContent = () => {
  const { address: account, isConnected } = useAccount();

  // Contract write hooks
  const { writeContractAsync: writeOwnershipContract } = useScaffoldWriteContract({
    contractName: "TrueOwnership",
  });

  const { writeContractAsync: writeAuthenticityContract } = useScaffoldWriteContract({
    contractName: "TrueAuthenticity",
  });

  // Read user registration status
  const { data: userInfo, refetch: refetchUserInfo } = useScaffoldReadContract({
    contractName: "TrueOwnership",
    functionName: "getUser",
    args: account ? [account] : undefined,
  });

  // Read manufacturer registration status
  // const { data: manufacturerInfo } = useScaffoldReadContract({
  //   contractName: "TrueAuthenticity",
  //   functionName: "getManufacturer",
  //   args: account ? [account] : undefined,
  // });

  const isUserRegistered = userInfo?.[0] || false;
  const userRegisteredName = userInfo?.[1] || "";
  // const isManufacturerRegistered = !!manufacturerInfo?.[0];
  // const manufacturerRegisteredName = manufacturerInfo?.[0] || "";

  const [activeTab, setActiveTab] = useState("overview");
  const [username, setUsername] = useState("");
  const [activeTransferCodes, setActiveTransferCodes] = useState<TransferCodeData[]>([]);
  const searchParams = useSearchParams();

  const [verificationData, setVerificationData] = useState({
    name: "",
    uniqueId: "",
    serial: "",
    date: "",
    owner: "",
    metadata: "",
    signature: "",
  });

  const [transferData, setTransferData] = useState({
    itemId: "",
    tempOwnerAddress: "",
  });

  const [claimData, setClaimData] = useState({
    ownershipCode: "",
  });

  // Read user's items using scaffold hook
  const {
    data: userItemsData,
    refetch: refetchMyItems,
    isLoading: isLoadingUserItems,
  } = useScaffoldReadContract({
    contractName: "TrueOwnership",
    functionName: "getAllMyItems",
    args: account && isUserRegistered ? [] : undefined,
  });

  const myItems = userItemsData || [];

  const loadMyItems = useCallback(async () => {
    if (account && isUserRegistered) {
      try {
        await refetchMyItems();
      } catch (error: any) {
        console.error("Error loading items:", error);
        // Don't show error toast for "no items found" as it's expected
        if (!error.message?.toLowerCase().includes("noitemsfound")) {
          toast.error(`Failed to load items: ${parseError(error)}`);
        }
      }
    }
  }, [account, isUserRegistered, refetchMyItems]);

  const loadActiveTransferCodes = useCallback(async () => {
    if (!account) return;

    try {
      const codes = await getActiveTransferCodes(account);
      setActiveTransferCodes(codes);
    } catch (error) {
      console.error("Error loading transfer codes:", error);
    }
  }, [account]);

  // Load user's items when account changes or when user is registered
  useEffect(() => {
    if (account && isUserRegistered) {
      loadMyItems();
      loadActiveTransferCodes();
    }
  }, [account, isUserRegistered, loadMyItems, loadActiveTransferCodes]);

  // Handle navigation state for auto-filling claim form
  useEffect(() => {
    const activeTabParam = searchParams.get("activeTab");
    const ownershipCodeParam = searchParams.get("ownershipCode");

    if (activeTabParam) {
      setActiveTab(activeTabParam);
    }
    if (ownershipCodeParam) {
      setClaimData({ ownershipCode: ownershipCodeParam });
    }
  }, [searchParams]);

  const registerUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim()) {
      toast.error("Please enter a username");
      return;
    }

    try {
      if (username.length < 3) {
        throw new Error("Username must be at least 3 characters");
      }

      await writeOwnershipContract({
        functionName: "userRegisters",
        args: [username],
      });

      // Refresh registration status
      await refetchUserInfo();

      toast.success(`User "${username}" registered successfully!`);
      setUsername("");
    } catch (error: any) {
      toast.error(`Registration failed: ${parseError(error)}`);
    }
  };

  const verifyProductAuthenticity = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account) {
      toast.error("Please connect your wallet");
      return;
    }

    try {
      if (
        !verificationData.name ||
        !verificationData.uniqueId ||
        !verificationData.serial ||
        !verificationData.date ||
        !verificationData.owner ||
        !verificationData.metadata ||
        !verificationData.signature
      ) {
        throw new Error("All fields are required for verification");
      }

      const metadata = verificationData.metadata
        .split(",")
        .map(item => item.trim())
        .filter(Boolean);

      const cert = {
        name: verificationData.name,
        uniqueId: verificationData.uniqueId,
        serial: verificationData.serial,
        date: parseInt(verificationData.date),
        owner: verificationData.owner,
        metadataHash: ethers.keccak256(ethers.AbiCoder.defaultAbiCoder().encode(["string[]"], [metadata])),
        metadata,
      };

      // For read operations, we can use useScaffoldReadContract with manual trigger
      // Since this is a one-time verification, we'll use the write contract hook
      await writeAuthenticityContract({
        functionName: "verifyAuthenticity",
        args: [cert, verificationData.signature],
      });

      toast.success(`Product "${verificationData.name}" verification completed!`);

      setVerificationData({
        name: "",
        uniqueId: "",
        serial: "",
        date: "",
        owner: "",
        metadata: "",
        signature: "",
      });
    } catch (error: any) {
      toast.error(`Verification failed: ${parseError(error)}`);
    }
  };

  const claimOwnership = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account) {
      toast.error("Please connect your wallet");
      return;
    }

    try {
      if (!verificationData.name || !verificationData.uniqueId || !verificationData.signature) {
        throw new Error("Product name, unique ID, and signature are required");
      }

      const metadata = verificationData.metadata
        .split(",")
        .map(item => item.trim())
        .filter(Boolean);

      const cert = {
        name: verificationData.name,
        uniqueId: verificationData.uniqueId,
        serial: verificationData.serial,
        date: parseInt(verificationData.date),
        owner: verificationData.owner,
        metadataHash: ethers.keccak256(ethers.AbiCoder.defaultAbiCoder().encode(["string[]"], [metadata])),
        metadata,
      };

      await writeAuthenticityContract({
        functionName: "userClaimOwnership",
        args: [cert, verificationData.signature],
      });

      toast.success("Ownership claimed successfully!");

      // Refresh items list
      await loadMyItems();

      setVerificationData({
        name: "",
        uniqueId: "",
        serial: "",
        date: "",
        owner: "",
        metadata: "",
        signature: "",
      });
    } catch (error: any) {
      toast.error(`Claim failed: ${parseError(error)}`);
    }
  };

  const generateTransferCode = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account) {
      toast.error("Please connect your wallet");
      return;
    }

    try {
      if (!transferData.itemId || !transferData.tempOwnerAddress) {
        throw new Error("Item ID and temporary owner address are required");
      }

      if (!ethers.isAddress(transferData.tempOwnerAddress)) {
        throw new Error("Invalid temporary owner address");
      }

      await writeOwnershipContract({
        functionName: "generateChangeOfOwnershipCode",
        args: [transferData.itemId, transferData.tempOwnerAddress],
      });

      // Note: We would need to listen for events to get the ownership code
      // For now, we'll show success without the specific code
      // In a full implementation, you'd use useScaffoldEventHistory or similar

      // Find the item details
      const item = myItems.find((item: any) => item.itemId === transferData.itemId);

      // Create notifications
      await Promise.all([
        // Notification for the recipient (User B)
        createNotification({
          user_address: transferData.tempOwnerAddress.toLowerCase(),
          type: "transfer_code_generated",
          title: "New Ownership Transfer",
          message: `You have received ownership transfer for "${item?.name || "Unknown Item"}"`,
          item_id: transferData.itemId,
          item_name: item?.name || "Unknown Item",
          from_address: account?.toLowerCase() || "",
          to_address: transferData.tempOwnerAddress.toLowerCase(),
          ownership_code: "pending", // Will be updated when event is processed
        }),
        // Notification for the sender (User A)
        createNotification({
          user_address: account?.toLowerCase() || "",
          type: "transfer_code_generated",
          title: "Transfer Code Generated",
          message: `Transfer code created for "${item?.name || "Unknown Item"}"`,
          item_id: transferData.itemId,
          item_name: item?.name || "Unknown Item",
          from_address: account?.toLowerCase() || "",
          to_address: transferData.tempOwnerAddress.toLowerCase(),
          ownership_code: "pending", // Will be updated when event is processed
        }),
      ]);

      toast.success(`Transfer code generated successfully!`);

      // Refresh active transfer codes
      await loadActiveTransferCodes();

      setTransferData({
        itemId: "",
        tempOwnerAddress: "",
      });
    } catch (error: any) {
      // Handle specific blockchain errors
      if (error.message?.includes("already generated") || error.message?.includes("code exists")) {
        toast.error(
          "A transfer code for this item and recipient already exists. Please revoke the existing code first or use a different recipient.",
        );
      } else {
        toast.error(`Transfer code generation failed: ${parseError(error)}`);
      }
    }
  };

  const handleRevokeTransferCode = async (transferCode: TransferCodeData) => {
    if (!account) {
      toast.error("Please connect your wallet");
      return;
    }

    try {
      // Revoke on blockchain
      await writeOwnershipContract({
        functionName: "ownerRevokeCode",
        args: [transferCode.ownership_code],
      });

      // Revoke in Supabase
      await revokeTransferCode(transferCode.ownership_code);

      // Create notifications
      await Promise.all([
        // Notification for the recipient (User B)
        createNotification({
          user_address: transferCode.to_address,
          type: "transfer_code_revoked",
          title: "Transfer Code Revoked",
          message: `Transfer code for "${transferCode.item_name}" has been revoked`,
          item_id: transferCode.item_id,
          item_name: transferCode.item_name,
          from_address: transferCode.from_address,
          to_address: transferCode.to_address,
        }),
        // Notification for the sender (User A)
        createNotification({
          user_address: account?.toLowerCase() || "",
          type: "transfer_code_revoked",
          title: "Transfer Code Revoked",
          message: `You revoked the transfer code for "${transferCode.item_name}"`,
          item_id: transferCode.item_id,
          item_name: transferCode.item_name,
          from_address: transferCode.from_address,
          to_address: transferCode.to_address,
        }),
      ]);

      toast.success("Transfer code revoked successfully!");

      // Refresh active transfer codes
      await loadActiveTransferCodes();
    } catch (error: any) {
      toast.error(`Revoke failed: ${parseError(error)}`);
    }
  };

  const claimWithCode = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account) {
      toast.error("Please connect your wallet");
      return;
    }

    try {
      if (!claimData.ownershipCode || !ethers.isBytesLike(claimData.ownershipCode)) {
        throw new Error("Valid ownership code required");
      }

      await writeOwnershipContract({
        functionName: "newOwnerClaimOwnership",
        args: [claimData.ownershipCode],
      });

      // Create notification for successful claim
      try {
        // Try to get transfer code details for better notification
        const transferCodeDetails = activeTransferCodes.find(code => code.ownership_code === claimData.ownershipCode);

        if (transferCodeDetails) {
          await createNotification({
            user_address: transferCodeDetails.from_address,
            type: "ownership_claimed",
            title: "Ownership Claimed",
            message: `"${transferCodeDetails.item_name}" ownership has been claimed`,
            item_id: transferCodeDetails.item_id,
            item_name: transferCodeDetails.item_name,
            from_address: transferCodeDetails.from_address,
            to_address: account?.toLowerCase() || "",
          });
        }
      } catch (notificationError) {
        console.error("Error creating claim notification:", notificationError);
      }

      toast.success("Ownership claimed with code successfully!");

      // Refresh items list
      await loadMyItems();

      setClaimData({ ownershipCode: "" });
    } catch (error: any) {
      toast.error(`Claim failed: ${parseError(error)}`);
    }
  };

  const tabs = [
    { id: "overview", label: "Overview", icon: User },
    { id: "register", label: "Register", icon: Plus },
    { id: "verify", label: "Verify Product", icon: Shield },
    { id: "claim", label: "Claim Ownership", icon: Key },
    { id: "transfer", label: "Transfer Ownership", icon: ArrowRightLeft },
    { id: "revoke", label: "Revoke Transfer", icon: X },
    { id: "my-items", label: "My Items", icon: Package },
  ];

  if (!account || !isConnected) {
    return (
      <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-orange-50 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center transition-colors duration-300">
        <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8 max-w-md w-full mx-4 transition-colors duration-300">
          <div className="text-center">
            <div className="w-16 h-16 bg-gradient-to-br from-blue-500 to-blue-600 rounded-full flex items-center justify-center mx-auto mb-6">
              <User className="h-8 w-8 text-white" />
            </div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">User Dashboard</h2>
            <p className="text-gray-600 dark:text-gray-300 mb-8">Connect your wallet to access user features</p>
            <div className="flex justify-center">
              <ConnectButton />
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-orange-50 dark:from-slate-900 dark:to-slate-800 transition-colors duration-300">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="bg-white dark:bg-slate-800 rounded-2xl shadow-lg p-6 mb-8 transition-colors duration-300">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl flex items-center justify-center">
                <User className="h-6 w-6 text-white" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white">User Dashboard</h1>
                <p className="text-gray-600 dark:text-gray-300">
                  {userRegisteredName ? (
                    <span>
                      Welcome back,{" "}
                      <span className="font-semibold text-blue-600 dark:text-blue-400">{userRegisteredName}</span>!
                    </span>
                  ) : (
                    `Connected: ${account.slice(0, 6)}...${account.slice(-4)}`
                  )}
                </p>
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <div
                className={`px-3 py-1 rounded-full text-sm font-medium ${
                  isUserRegistered
                    ? "bg-green-100 dark:bg-green-900/50 text-green-700 dark:text-green-300"
                    : "bg-yellow-100 dark:bg-yellow-900/50 text-yellow-700 dark:text-yellow-300"
                }`}
              >
                {isUserRegistered ? "Registered" : "Not Registered"}
              </div>
            </div>
          </div>
        </div>

        <div className="grid lg:grid-cols-4 gap-8">
          {/* Sidebar */}
          <div className="lg:col-span-1">
            <div className="bg-white dark:bg-slate-800 rounded-2xl shadow-lg p-6 transition-colors duration-300">
              <nav className="space-y-2">
                {tabs.map(tab => (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`w-full flex items-center space-x-3 px-4 py-3 rounded-lg text-left transition-all duration-300 ${
                      activeTab === tab.id
                        ? "bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 font-medium"
                        : "text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 hover:text-blue-600 dark:hover:text-blue-400"
                    }`}
                  >
                    <tab.icon className="h-5 w-5" />
                    <span>{tab.label}</span>
                  </button>
                ))}
              </nav>
            </div>
          </div>

          {/* Main Content */}
          <div className="lg:col-span-3">
            <div className="bg-white dark:bg-slate-800 rounded-2xl shadow-lg p-8 transition-colors duration-300">
              {/* Overview Tab */}
              {activeTab === "overview" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Overview</h2>
                  <div className="grid md:grid-cols-3 gap-6 mb-8">
                    <div className="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white">
                      <Package className="h-8 w-8 mb-4" />
                      <h3 className="text-lg font-semibold mb-2">Owned Items</h3>
                      <p className="text-3xl font-bold">{myItems.length}</p>
                    </div>
                    <div className="bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-xl p-6 text-white">
                      <Shield className="h-8 w-8 mb-4" />
                      <h3 className="text-lg font-semibold mb-2">Verified Products</h3>
                      <p className="text-3xl font-bold">0</p>
                    </div>
                    <div className="bg-gradient-to-br from-orange-500 to-orange-600 rounded-xl p-6 text-white">
                      <ArrowRightLeft className="h-8 w-8 mb-4" />
                      <h3 className="text-lg font-semibold mb-2">Transfers</h3>
                      <p className="text-3xl font-bold">0</p>
                    </div>
                  </div>

                  <div className="bg-gray-50 dark:bg-slate-700 rounded-xl p-6 transition-colors duration-300">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Actions</h3>
                    <div className="grid md:grid-cols-2 gap-4">
                      <button
                        onClick={() => setActiveTab("verify")}
                        className="flex items-center space-x-3 p-4 rounded-lg border-2 border-dashed border-blue-300 dark:border-blue-600 text-blue-600 dark:text-blue-400 hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-all duration-300"
                      >
                        <Scan className="h-5 w-5" />
                        <span>Verify Product</span>
                      </button>
                      <button
                        onClick={() => setActiveTab("claim")}
                        className="flex items-center space-x-3 p-4 rounded-lg border-2 border-dashed border-emerald-300 dark:border-emerald-600 text-emerald-600 dark:text-emerald-400 hover:border-emerald-400 dark:hover:border-emerald-500 hover:bg-emerald-50 dark:hover:bg-emerald-900/30 transition-all duration-300"
                      >
                        <Key className="h-5 w-5" />
                        <span>Claim Ownership</span>
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {/* Register Tab */}
              {activeTab === "register" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Register as User</h2>
                  {!isUserRegistered ? (
                    <form onSubmit={registerUser} className="space-y-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Username
                        </label>
                        <input
                          type="text"
                          value={username}
                          onChange={e => setUsername(e.target.value)}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                          placeholder="Enter your username (min 3 characters)"
                          minLength={3}
                          required
                        />
                      </div>
                      <button
                        type="submit"
                        className="w-full bg-gradient-to-r from-blue-600 to-blue-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-blue-700 hover:to-blue-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                      >
                        Register User
                      </button>
                    </form>
                  ) : (
                    <div className="text-center py-12">
                      <CheckCircle className="h-16 w-16 text-green-500 mx-auto mb-4" />
                      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">Already Registered</h3>
                      <p className="text-gray-600 dark:text-gray-300">You are successfully registered as a user.</p>
                    </div>
                  )}
                </div>
              )}

              {/* Verify Product Tab */}
              {activeTab === "verify" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Verify Product Authenticity</h2>
                  <form onSubmit={verifyProductAuthenticity} className="space-y-6">
                    <div className="grid md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Product Name
                        </label>
                        <input
                          type="text"
                          value={verificationData.name}
                          onChange={e => setVerificationData({ ...verificationData, name: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                          placeholder="e.g., iPhone 15 Pro"
                          required
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Unique ID
                        </label>
                        <input
                          type="text"
                          value={verificationData.uniqueId}
                          onChange={e => setVerificationData({ ...verificationData, uniqueId: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                          placeholder="Product unique identifier"
                          required
                        />
                      </div>
                    </div>

                    <div className="grid md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Serial Number
                        </label>
                        <input
                          type="text"
                          value={verificationData.serial}
                          onChange={e => setVerificationData({ ...verificationData, serial: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                          placeholder="Product serial number"
                          required
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Date (Unix timestamp)
                        </label>
                        <input
                          type="number"
                          value={verificationData.date}
                          onChange={e => setVerificationData({ ...verificationData, date: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                          placeholder="Manufacturing date"
                          required
                        />
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Owner Address
                      </label>
                      <input
                        type="text"
                        value={verificationData.owner}
                        onChange={e => setVerificationData({ ...verificationData, owner: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                        placeholder="Product owner address"
                        required
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Metadata (comma-separated)
                      </label>
                      <input
                        type="text"
                        value={verificationData.metadata}
                        onChange={e => setVerificationData({ ...verificationData, metadata: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                        placeholder="e.g., Black, 128GB, Pro Model"
                        required
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Signature
                      </label>
                      <textarea
                        value={verificationData.signature}
                        onChange={e => setVerificationData({ ...verificationData, signature: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors duration-300"
                        placeholder="Product signature from manufacturer"
                        rows={3}
                        required
                      />
                    </div>

                    <button
                      type="submit"
                      className="w-full bg-gradient-to-r from-blue-600 to-blue-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-blue-700 hover:to-blue-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                    >
                      Verify Product Authenticity
                    </button>
                  </form>
                </div>
              )}

              {/* Claim Ownership Tab */}
              {activeTab === "claim" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Claim Product Ownership</h2>

                  <div className="mb-8">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                      Method 1: Claim with Product Details
                    </h3>
                    <form onSubmit={claimOwnership} className="space-y-6">
                      <div className="grid md:grid-cols-2 gap-6">
                        <div>
                          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Product Name
                          </label>
                          <input
                            type="text"
                            value={verificationData.name}
                            onChange={e => setVerificationData({ ...verificationData, name: e.target.value })}
                            className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                            placeholder="e.g., iPhone 15 Pro"
                            required
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            Unique ID
                          </label>
                          <input
                            type="text"
                            value={verificationData.uniqueId}
                            onChange={e => setVerificationData({ ...verificationData, uniqueId: e.target.value })}
                            className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                            placeholder="Product unique identifier"
                            required
                          />
                        </div>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Signature
                        </label>
                        <textarea
                          value={verificationData.signature}
                          onChange={e => setVerificationData({ ...verificationData, signature: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                          placeholder="Product signature from manufacturer"
                          rows={3}
                          required
                        />
                      </div>

                      <button
                        type="submit"
                        className="w-full bg-gradient-to-r from-emerald-600 to-emerald-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-emerald-700 hover:to-emerald-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                      >
                        Claim Ownership
                      </button>
                    </form>
                  </div>

                  <div className="border-t border-gray-200 dark:border-gray-600 pt-8">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                      Method 2: Claim with Ownership Code
                    </h3>
                    <form onSubmit={claimWithCode} className="space-y-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Ownership Code
                        </label>
                        <input
                          type="text"
                          value={claimData.ownershipCode}
                          onChange={e => setClaimData({ ...claimData, ownershipCode: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-colors duration-300"
                          placeholder="Enter ownership transfer code"
                          required
                        />
                      </div>

                      <button
                        type="submit"
                        className="w-full bg-gradient-to-r from-orange-600 to-orange-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-orange-700 hover:to-orange-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                      >
                        Claim with Code
                      </button>
                    </form>
                  </div>
                </div>
              )}

              {/* Transfer Ownership Tab */}
              {activeTab === "transfer" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Transfer Ownership</h2>
                  <div className="bg-blue-50 dark:bg-slate-700/30 border border-blue-200 dark:border-slate-600 rounded-xl p-4 mb-6">
                    <div className="flex items-start space-x-3">
                      <ArrowRightLeft className="h-5 w-5 text-blue-600 dark:text-blue-400 mt-0.5" />
                      <div>
                        <h4 className="font-medium text-blue-900 dark:text-blue-300 mb-1">Transfer Process</h4>
                        <p className="text-sm text-blue-700 dark:text-blue-400">
                          Generate a transfer code to securely transfer ownership to another user. The recipient will
                          receive a notification and can claim ownership using the code.
                        </p>
                      </div>
                    </div>
                  </div>
                  <form onSubmit={generateTransferCode} className="space-y-6">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Select Item to Transfer
                      </label>
                      <div className="relative">
                        <select
                          value={transferData.itemId}
                          onChange={e => setTransferData({ ...transferData, itemId: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 transition-colors duration-300 appearance-none"
                          required
                        >
                          <option value="">Select an item you own</option>
                          {myItems.map((item: any, index: number) => (
                            <option key={index} value={item.itemId}>
                              {item.name} - {item.itemId}
                            </option>
                          ))}
                        </select>
                        <ChevronDown className="absolute right-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-gray-400 pointer-events-none" />
                      </div>
                      {myItems.length === 0 && (
                        <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                          {isLoadingUserItems
                            ? "Loading your items..."
                            : "You don&apos;t own any items yet. Claim some products first."}
                        </p>
                      )}
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        New Owner Address
                      </label>
                      <input
                        type="text"
                        value={transferData.tempOwnerAddress}
                        onChange={e => setTransferData({ ...transferData, tempOwnerAddress: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 transition-colors duration-300"
                        placeholder="Enter new owner's wallet address"
                        required
                      />
                    </div>

                    <button
                      type="submit"
                      disabled={myItems.length === 0}
                      className="w-full bg-gradient-to-r from-purple-600 to-purple-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-purple-700 hover:to-purple-800 transition-all duration-300 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Generate Transfer Code
                    </button>
                  </form>
                </div>
              )}

              {/* Revoke Transfer Tab */}
              {activeTab === "revoke" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Revoke Transfer Codes</h2>
                  <div className="bg-red-50 dark:bg-slate-700/30 border border-red-200 dark:border-slate-600 rounded-xl p-4 mb-6">
                    <div className="flex items-start space-x-3">
                      <X className="h-5 w-5 text-red-600 dark:text-red-400 mt-0.5" />
                      <div>
                        <h4 className="font-medium text-red-900 dark:text-red-300 mb-1">Revoke Transfer Codes</h4>
                        <p className="text-sm text-red-700 dark:text-red-400">
                          Manage and revoke active transfer codes. Once revoked, the recipient will no longer be able to
                          claim ownership.
                        </p>
                      </div>
                    </div>
                  </div>

                  {/* Active Transfer Codes */}
                  {activeTransferCodes.length > 0 ? (
                    <div className="space-y-4">
                      {activeTransferCodes.map(transferCode => (
                        <div
                          key={transferCode.id}
                          className="bg-white dark:bg-slate-700 border border-gray-200 dark:border-slate-600 rounded-xl p-6 transition-colors duration-300"
                        >
                          <div className="flex items-start justify-between mb-4">
                            <div className="flex-1">
                              <h4 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                                {transferCode.item_name}
                              </h4>
                              <div className="grid md:grid-cols-2 gap-4 text-sm">
                                <div>
                                  <p className="text-gray-600 dark:text-slate-300 mb-1">
                                    <span className="font-medium">Item ID:</span> {transferCode.item_id}
                                  </p>
                                  <p className="text-gray-600 dark:text-slate-300">
                                    <span className="font-medium">Recipient:</span>{" "}
                                    {transferCode.to_address.slice(0, 6)}...{transferCode.to_address.slice(-4)}
                                  </p>
                                </div>
                                <div>
                                  <p className="text-gray-600 dark:text-slate-300 mb-1">
                                    <span className="font-medium">Created:</span>{" "}
                                    {new Date(transferCode.created_at).toLocaleDateString()}
                                  </p>
                                  <p className="text-gray-600 dark:text-slate-300">
                                    <span className="font-medium">Expires:</span>{" "}
                                    {new Date(transferCode.expires_at).toLocaleDateString()}
                                  </p>
                                </div>
                              </div>
                            </div>
                            <div className="ml-4">
                              <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-green-100 dark:bg-green-900/50 text-green-700 dark:text-green-300 mb-3">
                                Active
                              </span>
                            </div>
                          </div>

                          <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-slate-600">
                            <div className="text-xs text-gray-500 dark:text-slate-400">
                              Transfer code: {transferCode.ownership_code.slice(0, 10)}...
                              {transferCode.ownership_code.slice(-6)}
                            </div>
                            <button
                              onClick={() => handleRevokeTransferCode(transferCode)}
                              className="inline-flex items-center space-x-2 bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700 transition-colors duration-300 text-sm font-medium"
                            >
                              <X className="h-4 w-4" />
                              <span>Revoke Code</span>
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-12">
                      <Key className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                        No Active Transfer Codes
                      </h3>
                      <p className="text-gray-600 dark:text-slate-300 mb-6">
                        You don&apos;t have any active transfer codes to revoke.
                      </p>
                      <button
                        onClick={() => setActiveTab("transfer")}
                        className="inline-flex items-center space-x-2 bg-blue-600 text-white px-6 py-3 rounded-lg hover:bg-blue-700 transition-colors duration-300"
                      >
                        <ArrowRightLeft className="h-4 w-4" />
                        <span>Create Transfer Code</span>
                      </button>
                    </div>
                  )}
                </div>
              )}

              {/* My Items Tab */}
              {activeTab === "my-items" && (
                <div>
                  <div className="flex items-center justify-between mb-6">
                    <h2 className="text-2xl font-bold text-gray-900 dark:text-white">My Items</h2>
                    <button
                      onClick={loadMyItems}
                      disabled={isLoadingUserItems}
                      className="inline-flex items-center space-x-2 bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors duration-300 disabled:opacity-50"
                    >
                      <Search className="h-4 w-4" />
                      <span>{isLoadingUserItems ? "Loading..." : "Refresh"}</span>
                    </button>
                  </div>

                  {isLoadingUserItems ? (
                    <div className="text-center py-12">
                      <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
                      <p className="text-gray-600 dark:text-gray-300">Loading your items...</p>
                    </div>
                  ) : myItems.length > 0 ? (
                    <div className="grid gap-6">
                      {myItems.map((item: any, index: number) => (
                        <div
                          key={index}
                          className="bg-gray-50 dark:bg-slate-700 rounded-xl p-6 border border-gray-200 dark:border-slate-600 transition-colors duration-300"
                        >
                          <div className="flex items-start justify-between mb-4">
                            <div>
                              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">{item.name}</h3>
                              <p className="text-sm text-gray-600 dark:text-gray-300">
                                Manufactured by: <span className="font-medium">{item.manufacturer}</span>
                              </p>
                            </div>
                            <div className="text-right">
                              <p className="text-sm text-gray-500 dark:text-gray-400">
                                Owned since: {new Date(Number(item.date) * 1000).toLocaleDateString()}
                              </p>
                            </div>
                          </div>

                          <div className="grid md:grid-cols-2 gap-4 mb-4">
                            <div>
                              <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Item ID</p>
                              <p className="text-sm text-gray-600 dark:text-gray-400 font-mono bg-white dark:bg-slate-600 px-3 py-2 rounded border">
                                {item.itemId}
                              </p>
                            </div>
                            <div>
                              <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Serial Number</p>
                              <p className="text-sm text-gray-600 dark:text-gray-400 font-mono bg-white dark:bg-slate-600 px-3 py-2 rounded border">
                                {item.serial}
                              </p>
                            </div>
                          </div>

                          {item.metadata && item.metadata.length > 0 && (
                            <div className="mb-4">
                              <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Product Details
                              </p>
                              <div className="flex flex-wrap gap-2">
                                {item.metadata.map((meta: string, metaIndex: number) => (
                                  <span
                                    key={metaIndex}
                                    className="px-3 py-1 bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 rounded-full text-sm"
                                  >
                                    {meta}
                                  </span>
                                ))}
                              </div>
                            </div>
                          )}

                          <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-slate-600">
                            <div className="flex items-center space-x-2">
                              <CheckCircle className="h-5 w-5 text-green-500" />
                              <span className="text-sm font-medium text-green-700 dark:text-green-400">
                                Verified & Owned
                              </span>
                            </div>
                            <button
                              onClick={() => {
                                setTransferData({ ...transferData, itemId: item.itemId });
                                setActiveTab("transfer");
                              }}
                              className="inline-flex items-center space-x-2 text-purple-600 dark:text-purple-400 hover:text-purple-700 dark:hover:text-purple-300 transition-colors duration-300"
                            >
                              <ArrowRightLeft className="h-4 w-4" />
                              <span>Transfer</span>
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-12">
                      <Package className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">No Items Found</h3>
                      <p className="text-gray-600 dark:text-gray-300 mb-6">
                        You don&apos;t own any verified items yet. Start by claiming ownership of a product.
                      </p>
                      <button
                        onClick={() => setActiveTab("claim")}
                        className="inline-flex items-center space-x-2 bg-blue-600 text-white px-6 py-3 rounded-lg hover:bg-blue-700 transition-colors duration-300"
                      >
                        <Plus className="h-4 w-4" />
                        <span>Claim Your First Item</span>
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

const UserDashboard = () => {
  return (
    <Suspense
      fallback={
        <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-orange-50 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center transition-colors duration-300">
          <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8 max-w-md w-full mx-4 transition-colors duration-300">
            <div className="text-center">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
              <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">Loading Dashboard</h2>
              <p className="text-gray-600 dark:text-gray-300">Please wait while we load your user dashboard...</p>
            </div>
          </div>
        </div>
      }
    >
      <UserDashboardContent />
    </Suspense>
  );
};

export default UserDashboard;
