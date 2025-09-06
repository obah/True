"use client";

import React, { useState } from "react";
import { parseError, signTypedData } from "../../utils/constants/blockchain";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import { ethers } from "ethers";
import { CheckCircle, Download, Eye, Factory, Package, Plus, QrCode, Shield } from "lucide-react";
import { QRCodeCanvas } from "qrcode.react";
import { toast } from "react-toastify";
import { useAccount, useWalletClient } from "wagmi";
import { useScaffoldReadContract, useScaffoldWriteContract } from "~~/hooks/scaffold-eth";

const ManufacturerDashboard = () => {
  const { address: account, isConnected, chain } = useAccount();
  const { data: walletClient } = useWalletClient();
  const chainId = chain?.id || 0;

  // Contract write hooks
  const { writeContractAsync: writeAuthenticityContract } = useScaffoldWriteContract({
    contractName: "TrueAuthenticity",
  });

  // Read manufacturer registration status
  const { data: manufacturerInfo, refetch: refetchManufacturerInfo } = useScaffoldReadContract({
    contractName: "TrueAuthenticity",
    functionName: "getManufacturer",
    args: account ? [account] : undefined,
  });

  const isManufacturerRegistered = !!manufacturerInfo?.[0];
  const manufacturerRegisteredName = manufacturerInfo?.[0] || "";

  const [activeTab, setActiveTab] = useState("overview");
  const [manufacturerName, setManufacturerName] = useState("");
  const [qrCodeData, setQrCodeData] = useState("");
  const [signature, setSignature] = useState("");

  const [certificate, setCertificate] = useState({
    name: "",
    uniqueId: "",
    serial: "",
    metadata: "",
  });

  const registerManufacturer = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account || !manufacturerName.trim()) {
      toast.error("Please enter a manufacturer name");
      return;
    }

    try {
      await writeAuthenticityContract({
        functionName: "manufacturerRegisters",
        args: [manufacturerName],
      });

      // Refresh registration status
      await refetchManufacturerInfo();

      toast.success(`Manufacturer "${manufacturerName}" registered successfully!`);
      setManufacturerName("");
    } catch (error: any) {
      toast.error(`Registration failed: ${parseError(error)}`);
    }
  };

  const createCertificate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!account || !walletClient) {
      toast.error("Please connect your wallet");
      return;
    }

    try {
      if (!certificate.name || !certificate.uniqueId || !certificate.serial || !certificate.metadata) {
        throw new Error("All certificate fields required");
      }

      const timestamp = Math.floor(Date.now() / 1000);
      const metadata = certificate.metadata
        .split(",")
        .map(item => item.trim())
        .filter(Boolean);

      const cert = {
        name: certificate.name,
        uniqueId: certificate.uniqueId,
        serial: certificate.serial,
        date: timestamp,
        owner: account,
        metadataHash: ethers.keccak256(ethers.AbiCoder.defaultAbiCoder().encode(["string[]"], [metadata])),
        metadata,
      };

      // Create typed data for signing
      const { domain, types, value } = signTypedData(cert, chainId);

      // Sign the certificate using wagmi wallet client
      const signedSignature = await walletClient.signTypedData({
        account,
        domain,
        types,
        primaryType: "Certificate",
        message: value,
      });
      setSignature(signedSignature);

      // Verify signature locally first
      const recoveredAddress = ethers.verifyTypedData(domain, types, value, signature);

      if (recoveredAddress.toLowerCase() !== account?.toLowerCase()) {
        throw new Error("Signature verification failed");
      }

      //   const qrData = JSON.stringify({ cert, signature });

      // Create URL with certificate data for QR code
      // Automatically detect the correct base URL
      // This will work for both development and production (Vercel)
      const baseUrl = window.location.origin;

      // Use a more robust URL encoding approach
      const params = new URLSearchParams();
      params.set("cert", JSON.stringify(cert));
      params.set("sig", signature);
      const verifyUrl = `${baseUrl}/verify?${params.toString()}`;

      console.log("Generated QR URL:", verifyUrl);

      setQrCodeData(verifyUrl);

      toast.success("Certificate created and signed successfully!");

      setCertificate({
        name: "",
        uniqueId: "",
        serial: "",
        metadata: "",
      });
    } catch (error: any) {
      toast.error(`Certificate creation failed: ${parseError(error)}`);
    }
  };

  const downloadQRCode = () => {
    const canvas = document.querySelector("#qr-code") as HTMLCanvasElement;
    if (canvas) {
      const link = document.createElement("a");
      link.href = canvas.toDataURL("image/png");
      link.download = `certificate-${certificate.uniqueId || "qr"}.png`;
      link.click();
    }
  };

  const tabs = [
    { id: "overview", label: "Overview", icon: Factory },
    { id: "register", label: "Register", icon: Plus },
    { id: "certificates", label: "Create Certificate", icon: Shield },
    { id: "verify", label: "Verify Products", icon: CheckCircle },
  ];

  if (!account || !isConnected) {
    return (
      <div className="pt-16 min-h-screen bg-gradient-to-br from-emerald-50 to-blue-50 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center transition-colors duration-300">
        <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-8 max-w-md w-full mx-4 transition-colors duration-300">
          <div className="text-center">
            <div className="w-16 h-16 bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-full flex items-center justify-center mx-auto mb-6">
              <Factory className="h-8 w-8 text-white" />
            </div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">Manufacturer Dashboard</h2>
            <p className="text-gray-600 dark:text-gray-300 mb-8">Connect your wallet to access manufacturer features</p>
            <div className="flex justify-center">
              <ConnectButton />
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="pt-16 min-h-screen bg-gradient-to-br from-emerald-50 to-blue-50 dark:from-slate-900 dark:to-slate-800 transition-colors duration-300">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="bg-white dark:bg-slate-800 rounded-2xl shadow-lg p-6 mb-8 transition-colors duration-300">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="w-12 h-12 bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-xl flex items-center justify-center">
                <Factory className="h-6 w-6 text-white" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-gray-900 dark:text-white">Manufacturer Dashboard</h1>
                <p className="text-gray-600 dark:text-gray-300">
                  {manufacturerRegisteredName ? (
                    <span>
                      Welcome back,{" "}
                      <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                        {manufacturerRegisteredName}
                      </span>
                      !
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
                  isManufacturerRegistered
                    ? "bg-green-100 dark:bg-green-900/50 text-green-700 dark:text-green-300"
                    : "bg-yellow-100 dark:bg-yellow-900/50 text-yellow-700 dark:text-yellow-300"
                }`}
              >
                {isManufacturerRegistered ? "Registered" : "Not Registered"}
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
                        ? "bg-emerald-100 dark:bg-emerald-900/50 text-emerald-700 dark:text-emerald-300 font-medium"
                        : "text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 hover:text-emerald-600 dark:hover:text-emerald-400"
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
                    <div className="bg-gradient-to-br from-emerald-500 to-emerald-600 rounded-xl p-6 text-white">
                      <Package className="h-8 w-8 mb-4" />
                      <h3 className="text-lg font-semibold mb-2">Products Created</h3>
                      <p className="text-3xl font-bold">0</p>
                    </div>
                    <div className="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white">
                      <QrCode className="h-8 w-8 mb-4" />
                      <h3 className="text-lg font-semibold mb-2">QR Codes Generated</h3>
                      <p className="text-3xl font-bold">0</p>
                    </div>
                    <div className="bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl p-6 text-white">
                      <CheckCircle className="h-8 w-8 mb-4" />
                      <h3 className="text-lg font-semibold mb-2">Verified Products</h3>
                      <p className="text-3xl font-bold">0</p>
                    </div>
                  </div>

                  <div className="bg-gray-50 dark:bg-slate-700 rounded-xl p-6 transition-colors duration-300">
                    <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">Quick Actions</h3>
                    <div className="grid md:grid-cols-2 gap-4">
                      <button
                        onClick={() => setActiveTab("register")}
                        disabled={isManufacturerRegistered}
                        className={`flex items-center space-x-3 p-4 rounded-lg border-2 border-dashed transition-all duration-300 ${
                          isManufacturerRegistered
                            ? "border-gray-200 dark:border-gray-600 text-gray-400 dark:text-gray-500 cursor-not-allowed"
                            : "border-emerald-300 dark:border-emerald-600 text-emerald-600 dark:text-emerald-400 hover:border-emerald-400 dark:hover:border-emerald-500 hover:bg-emerald-50 dark:hover:bg-emerald-900/30"
                        }`}
                      >
                        <Plus className="h-5 w-5" />
                        <span>{isManufacturerRegistered ? "Already Registered" : "Register as Manufacturer"}</span>
                      </button>
                      <button
                        onClick={() => setActiveTab("certificates")}
                        className="flex items-center space-x-3 p-4 rounded-lg border-2 border-dashed border-blue-300 dark:border-blue-600 text-blue-600 dark:text-blue-400 hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-all duration-300"
                      >
                        <Shield className="h-5 w-5" />
                        <span>Create New Certificate</span>
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {/* Register Tab */}
              {activeTab === "register" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Register as Manufacturer</h2>
                  {!isManufacturerRegistered ? (
                    <form onSubmit={registerManufacturer} className="space-y-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Manufacturer Name
                        </label>
                        <input
                          type="text"
                          value={manufacturerName}
                          onChange={e => setManufacturerName(e.target.value)}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                          placeholder="Enter your company name"
                          required
                        />
                      </div>
                      <button
                        type="submit"
                        className="w-full bg-gradient-to-r from-emerald-600 to-emerald-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-emerald-700 hover:to-emerald-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                      >
                        Register Manufacturer
                      </button>
                    </form>
                  ) : (
                    <div className="text-center py-12">
                      <CheckCircle className="h-16 w-16 text-green-500 mx-auto mb-4" />
                      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">Already Registered</h3>
                      <p className="text-gray-600 dark:text-gray-300">
                        You are successfully registered as a manufacturer.
                      </p>
                    </div>
                  )}
                </div>
              )}

              {/* Certificates Tab */}
              {activeTab === "certificates" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Create Product Certificate</h2>
                  <form onSubmit={createCertificate} className="space-y-6">
                    <div className="grid md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          Product Name
                        </label>
                        <input
                          type="text"
                          value={certificate.name}
                          onChange={e => setCertificate({ ...certificate, name: e.target.value })}
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
                          value={certificate.uniqueId}
                          onChange={e => setCertificate({ ...certificate, uniqueId: e.target.value })}
                          className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                          placeholder="e.g., IMEI, Serial Number"
                          required
                        />
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Serial Number
                      </label>
                      <input
                        type="text"
                        value={certificate.serial}
                        onChange={e => setCertificate({ ...certificate, serial: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                        placeholder="Product serial number"
                        required
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        Metadata (comma-separated)
                      </label>
                      <input
                        type="text"
                        value={certificate.metadata}
                        onChange={e => setCertificate({ ...certificate, metadata: e.target.value })}
                        className="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white rounded-lg focus:ring-2 focus:ring-emerald-500 focus:border-emerald-500 transition-colors duration-300"
                        placeholder="e.g., Black, 128GB, Pro Model"
                        required
                      />
                    </div>

                    <button
                      type="submit"
                      className="w-full bg-gradient-to-r from-emerald-600 to-emerald-700 text-white py-3 px-6 rounded-lg font-semibold hover:from-emerald-700 hover:to-emerald-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                    >
                      Create & Sign Certificate
                    </button>
                  </form>

                  {/* QR Code Display */}
                  {qrCodeData && (
                    <div className="mt-8 p-6 bg-gradient-to-br from-blue-50 to-emerald-50 dark:from-blue-900/20 dark:to-emerald-900/20 rounded-xl border border-blue-200 dark:border-blue-700 transition-colors duration-300">
                      <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 text-center">
                        Product Verification QR Code
                      </h3>
                      <p className="text-sm text-gray-600 dark:text-gray-300 text-center mb-6">
                        Scan this QR code to verify authenticity, claim ownership, or check ownership status
                      </p>
                      <div className="flex justify-center mb-4">
                        <QRCodeCanvas
                          id="qr-code"
                          value={qrCodeData}
                          size={256}
                          className="border-4 border-white rounded-xl shadow-lg"
                          level="M"
                          includeMargin={true}
                        />
                      </div>
                      <div className="text-center space-y-3">
                        <p className="text-xs text-gray-500 dark:text-gray-400 max-w-md mx-auto">
                          This QR code contains the product certificate and verification URL. Users can scan it to
                          instantly verify and claim the product.
                        </p>
                        <button
                          onClick={downloadQRCode}
                          className="inline-flex items-center space-x-2 bg-gradient-to-r from-blue-600 to-blue-700 text-white px-6 py-3 rounded-lg hover:from-blue-700 hover:to-blue-800 transition-all duration-300 shadow-lg hover:shadow-xl"
                        >
                          <Download className="h-4 w-4" />
                          <span>Download QR Code</span>
                        </button>
                      </div>

                      {/* URL Display */}
                      <div className="mt-6 p-4 bg-white dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600 transition-colors duration-300">
                        <p className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Verification URL:</p>
                        <div className="flex items-center space-x-2">
                          <input
                            type="text"
                            value={qrCodeData}
                            readOnly
                            className="flex-1 text-xs font-mono bg-gray-50 dark:bg-gray-600 px-3 py-2 rounded border dark:border-gray-500 text-gray-600 dark:text-gray-300"
                          />
                          <button
                            onClick={() => {
                              navigator.clipboard.writeText(qrCodeData);
                              toast.success("URL copied to clipboard!");
                            }}
                            className="px-3 py-2 bg-gray-100 dark:bg-gray-600 text-gray-600 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-300 text-sm"
                          >
                            Copy
                          </button>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              )}

              {/* Verify Tab */}
              {activeTab === "verify" && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-6">Verify Products</h2>
                  <div className="text-center py-12">
                    <Eye className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                    <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">Product Verification</h3>
                    <p className="text-gray-600 dark:text-gray-300">
                      Use this section to verify products and view verification history.
                    </p>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ManufacturerDashboard;
