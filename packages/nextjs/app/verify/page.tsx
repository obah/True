"use client";

import React, { Suspense, useEffect, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { parseError } from "../../utils/constants/blockchain";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import {
  AlertTriangle,
  ArrowLeft,
  Calendar,
  CheckCircle,
  Hash,
  Key,
  Package,
  Scan,
  Shield,
  User,
  XCircle,
} from "lucide-react";
import { toast } from "react-toastify";
import { useAccount } from "wagmi";
import { useScaffoldReadContract, useScaffoldWriteContract } from "~~/hooks/scaffold-eth";

const VerifyPageContent = () => {
  const searchParams = useSearchParams();
  const { address: account, isConnected } = useAccount();

  // Contract write hooks
  const { writeContractAsync: writeAuthenticityContract } = useScaffoldWriteContract({
    contractName: "TrueAuthenticity",
  });

  // State for verification call arguments
  const [verificationArgs, setVerificationArgs] = useState<[any, string] | undefined>(undefined);

  // Use scaffold read contract for verification when args are available
  const { data: verificationData } = useScaffoldReadContract({
    contractName: "TrueAuthenticity",
    functionName: "verifyAuthenticity",
    args: verificationArgs,
  });

  // State for ownership check arguments
  const [ownershipCheckId, setOwnershipCheckId] = useState<string | undefined>(undefined);

  // Use scaffold read contract for ownership check
  const { data: ownershipData } = useScaffoldReadContract({
    contractName: "TrueOwnership",
    functionName: "verifyOwnership",
    args: ownershipCheckId ? [ownershipCheckId] : undefined,
  });

  const [certificate, setCertificate] = useState<any>(null);
  const [signature, setSignature] = useState("");
  const [verificationResult, setVerificationResult] = useState<{
    isValid: boolean;
    manufacturerName: string;
  } | null>(null);
  const [ownershipInfo, setOwnershipInfo] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [activeAction, setActiveAction] = useState<"verify" | "claim" | "ownership">("verify");

  // Add debug logging
  useEffect(() => {
    const certData = searchParams.get("cert");
    const sigData = searchParams.get("sig");

    console.log("URL params:", { certData, sigData });
    console.log("Full URL:", window.location.href);

    if (certData && sigData) {
      try {
        const decodedCert = JSON.parse(decodeURIComponent(certData));
        const decodedSig = decodeURIComponent(sigData);

        console.log("Decoded certificate:", decodedCert);
        console.log("Decoded signature:", decodedSig);

        setCertificate(decodedCert);
        setSignature(decodedSig);

        // Auto-verify will happen in the verification effect
      } catch (error) {
        toast.error("Invalid QR code data");
        console.error("Error parsing QR data:", error);
      }
    } else {
      console.log("Missing URL parameters");
      if (!certData) console.log("Missing cert parameter");
      if (!sigData) console.log("Missing sig parameter");
    }

    setIsLoading(false);
  }, [searchParams]);

  // Auto-verification when certificate and signature are available
  useEffect(() => {
    if (certificate && signature && !verificationResult) {
      verifyAuthenticity(certificate, signature);
    }
  }, [certificate, signature]);

  // Update verification result when contract data changes
  useEffect(() => {
    if (verificationData) {
      const isValid = verificationData[0] || false;
      const manufacturerName = verificationData[1] || "";

      setVerificationResult({ isValid, manufacturerName });

      if (isValid) {
        toast.success(`Product is authentic! Manufactured by ${manufacturerName}`);
        // Also check ownership info
        if (certificate?.uniqueId) {
          setOwnershipCheckId(certificate.uniqueId);
        }
      } else {
        toast.error("Product authenticity could not be verified!");
      }
      setIsLoading(false);
    }
  }, [verificationData, certificate]);

  // Update ownership info when contract data changes
  useEffect(() => {
    if (ownershipData) {
      setOwnershipInfo(ownershipData);
    }
  }, [ownershipData]);

  const verifyAuthenticity = async (cert?: any, sig?: string) => {
    const targetCert = cert || certificate;
    const targetSig = sig || signature;

    if (!targetCert || !targetSig) {
      toast.error("Missing data for verification");
      return;
    }

    try {
      setIsLoading(true);

      // Set the verification arguments to trigger the useScaffoldReadContract
      setVerificationArgs([targetCert, targetSig]);

      // The actual verification result will be handled in the useEffect for verificationData
    } catch (error: any) {
      toast.error(`Verification failed: ${parseError(error)}`);
      setVerificationResult({ isValid: false, manufacturerName: "" });
      setIsLoading(false);
    }
  };

  //   const checkOwnership = async (itemId: string) => {
  //     try {
  //       // Set the ownership check ID to trigger the useScaffoldReadContract
  //       setOwnershipCheckId(itemId);
  //       console.log("Checking ownership for:", itemId);
  //     } catch (error) {
  //       // Item might not be claimed yet, which is fine
  //       console.log("Ownership not found:", error);
  //     }
  //   };

  const claimOwnership = async () => {
    if (!account) {
      toast.error("Please connect your wallet first");
      return;
    }

    if (!certificate || !signature) {
      toast.error("Missing certificate data");
      return;
    }

    try {
      setIsLoading(true);

      await writeAuthenticityContract({
        functionName: "userClaimOwnership",
        args: [certificate, signature],
      });

      toast.success("Ownership claimed successfully!");
      // Refresh ownership info by re-triggering the ownership check
      if (certificate.uniqueId) {
        setOwnershipCheckId(certificate.uniqueId);
      }
    } catch (error: any) {
      toast.error(`Claim failed: ${parseError(error)}`);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading && !certificate) {
    return (
      <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-emerald-50 flex items-center justify-center">
        <div className="bg-white rounded-2xl shadow-xl p-8 max-w-md w-full mx-4">
          <div className="text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
            <p className="text-gray-600">Loading certificate data...</p>
          </div>
        </div>
      </div>
    );
  }

  if (!certificate) {
    return (
      <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-emerald-50 flex items-center justify-center">
        <div className="bg-white rounded-2xl shadow-xl p-8 max-w-md w-full mx-4">
          <div className="text-center">
            <AlertTriangle className="h-16 w-16 text-yellow-500 mx-auto mb-4" />
            <h2 className="text-2xl font-bold text-gray-900 mb-4">Invalid QR Code</h2>
            <p className="text-gray-600 mb-6">The QR code you scanned doesn&apos;t contain valid certificate data.</p>
            <Link
              href="/"
              className="inline-flex items-center space-x-2 bg-blue-600 text-white px-6 py-3 rounded-lg hover:bg-blue-700 transition-colors duration-300"
            >
              <ArrowLeft className="h-4 w-4" />
              <span>Go Home</span>
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-emerald-50">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Header */}
        <div className="bg-white rounded-2xl shadow-lg p-6 mb-8">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className="w-12 h-12 bg-gradient-to-br from-blue-500 to-emerald-500 rounded-xl flex items-center justify-center">
                <Scan className="h-6 w-6 text-white" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-gray-900">Product Verification</h1>
                <p className="text-gray-600">Verify authenticity and ownership</p>
              </div>
            </div>
            <Link
              href="/"
              className="inline-flex items-center space-x-2 text-gray-600 hover:text-blue-600 transition-colors duration-300"
            >
              <ArrowLeft className="h-4 w-4" />
              <span>Back to Home</span>
            </Link>
          </div>
        </div>

        {/* Action Tabs */}
        <div className="bg-white rounded-2xl shadow-lg mb-8">
          <div className="border-b border-gray-200">
            <nav className="flex space-x-8 px-6">
              {[
                { id: "verify", label: "Verify Authenticity", icon: Shield },
                { id: "claim", label: "Claim Ownership", icon: Key },
                { id: "ownership", label: "Check Ownership", icon: User },
              ].map(tab => (
                <button
                  key={tab.id}
                  onClick={() => setActiveAction(tab.id as any)}
                  className={`flex items-center space-x-2 py-4 border-b-2 font-medium text-sm transition-colors duration-300 ${
                    activeAction === tab.id
                      ? "border-blue-500 text-blue-600"
                      : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                  }`}
                >
                  <tab.icon className="h-4 w-4" />
                  <span>{tab.label}</span>
                </button>
              ))}
            </nav>
          </div>

          <div className="p-6">
            {/* Verify Authenticity Tab */}
            {activeAction === "verify" && (
              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-6">Product Authenticity Verification</h3>

                {verificationResult ? (
                  <div
                    className={`rounded-xl p-6 mb-6 ${
                      verificationResult.isValid
                        ? "bg-green-50 border border-green-200"
                        : "bg-red-50 border border-red-200"
                    }`}
                  >
                    <div className="flex items-center space-x-3 mb-4">
                      {verificationResult.isValid ? (
                        <CheckCircle className="h-8 w-8 text-green-600" />
                      ) : (
                        <XCircle className="h-8 w-8 text-red-600" />
                      )}
                      <div>
                        <h4
                          className={`text-lg font-semibold ${
                            verificationResult.isValid ? "text-green-900" : "text-red-900"
                          }`}
                        >
                          {verificationResult.isValid ? "Product is Authentic" : "Product Authenticity Failed!"}
                        </h4>
                        {verificationResult.isValid && verificationResult.manufacturerName && (
                          <p className="text-green-700">Manufactured by: {verificationResult.manufacturerName}</p>
                        )}
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-8">
                    <button
                      onClick={() => verifyAuthenticity()}
                      disabled={isLoading}
                      className="bg-gradient-to-r from-blue-600 to-blue-700 text-white px-8 py-3 rounded-lg font-semibold hover:from-blue-700 hover:to-blue-800 transition-all duration-300 shadow-lg hover:shadow-xl disabled:opacity-50"
                    >
                      {isLoading ? "Verifying..." : "Verify Authenticity"}
                    </button>
                  </div>
                )}
              </div>
            )}

            {/* Claim Ownership Tab */}
            {activeAction === "claim" && (
              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-6">Claim Product Ownership</h3>

                {!account || !isConnected ? (
                  <div className="text-center py-8">
                    <User className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                    <h4 className="text-lg font-semibold text-gray-900 mb-2">Connect Your Wallet</h4>
                    <p className="text-gray-600 mb-6">
                      You need to connect your wallet to claim ownership of this product.
                    </p>
                    <div className="flex justify-center">
                      <ConnectButton />
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-8">
                    <Key className="h-16 w-16 text-emerald-500 mx-auto mb-4" />
                    <h4 className="text-lg font-semibold text-gray-900 mb-2">Claim This Product</h4>
                    <p className="text-gray-600 mb-6">Click below to claim ownership of this verified product.</p>
                    <button
                      onClick={claimOwnership}
                      disabled={isLoading}
                      className="bg-gradient-to-r from-emerald-600 to-emerald-700 text-white px-8 py-3 rounded-lg font-semibold hover:from-emerald-700 hover:to-emerald-800 transition-all duration-300 shadow-lg hover:shadow-xl disabled:opacity-50"
                    >
                      {isLoading ? "Claiming..." : "Claim Ownership"}
                    </button>
                  </div>
                )}
              </div>
            )}

            {/* Check Ownership Tab */}
            {activeAction === "ownership" && (
              <div>
                <h3 className="text-lg font-semibold text-gray-900 mb-6">Ownership Information</h3>

                {ownershipInfo ? (
                  <div className="bg-blue-50 border border-blue-200 rounded-xl p-6">
                    <div className="flex items-center space-x-3 mb-4">
                      <CheckCircle className="h-8 w-8 text-blue-600" />
                      <div>
                        <h4 className="text-lg font-semibold text-blue-900">Product is Owned</h4>
                        <p className="text-blue-700">This product has been claimed by a verified user.</p>
                      </div>
                    </div>
                    <div className="grid md:grid-cols-2 gap-4 mt-4">
                      <div>
                        <p className="text-sm font-medium text-blue-700 mb-1">Owner</p>
                        <p className="text-sm text-blue-600">{ownershipInfo.username}</p>
                      </div>
                      <div>
                        <p className="text-sm font-medium text-blue-700 mb-1">Owner Address</p>
                        <p className="text-sm text-blue-600 font-mono">
                          {ownershipInfo.owner.slice(0, 6)}...{ownershipInfo.owner.slice(-4)}
                        </p>
                      </div>
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-8">
                    <Package className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                    <h4 className="text-lg font-semibold text-gray-900 mb-2">No Owner Found</h4>
                    <p className="text-gray-600">This product has not been claimed by any user yet.</p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Product Details */}
        <div className="bg-white rounded-2xl shadow-lg p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-6">Product Details</h3>

          <div className="grid md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <div>
                <div className="flex items-center space-x-2 mb-2">
                  <Package className="h-4 w-4 text-gray-500" />
                  <p className="text-sm font-medium text-gray-700">Product Name</p>
                </div>
                <p className="text-gray-900 font-semibold">{certificate.name}</p>
              </div>

              <div>
                <div className="flex items-center space-x-2 mb-2">
                  <Hash className="h-4 w-4 text-gray-500" />
                  <p className="text-sm font-medium text-gray-700">Unique ID</p>
                </div>
                <p className="text-gray-900 font-mono text-sm bg-gray-50 px-3 py-2 rounded border">
                  {certificate.uniqueId}
                </p>
              </div>

              <div>
                <div className="flex items-center space-x-2 mb-2">
                  <Hash className="h-4 w-4 text-gray-500" />
                  <p className="text-sm font-medium text-gray-700">Serial Number</p>
                </div>
                <p className="text-gray-900 font-mono text-sm bg-gray-50 px-3 py-2 rounded border">
                  {certificate.serial}
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <div>
                <div className="flex items-center space-x-2 mb-2">
                  <Calendar className="h-4 w-4 text-gray-500" />
                  <p className="text-sm font-medium text-gray-700">Manufacturing Date</p>
                </div>
                <p className="text-gray-900">{new Date(Number(certificate.date) * 1000).toLocaleDateString()}</p>
              </div>

              <div>
                <div className="flex items-center space-x-2 mb-2">
                  <User className="h-4 w-4 text-gray-500" />
                  <p className="text-sm font-medium text-gray-700">Original Owner</p>
                </div>
                <p className="text-gray-900 font-mono text-sm">
                  {certificate.owner.slice(0, 6)}...{certificate.owner.slice(-4)}
                </p>
              </div>

              {certificate.metadata && certificate.metadata.length > 0 && (
                <div>
                  <div className="flex items-center space-x-2 mb-2">
                    <Package className="h-4 w-4 text-gray-500" />
                    <p className="text-sm font-medium text-gray-700">Product Details</p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {certificate.metadata.map((meta: string, index: number) => (
                      <span key={index} className="px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-sm">
                        {meta}
                      </span>
                    ))}
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

const VerifyPage = () => {
  return (
    <Suspense
      fallback={
        <div className="pt-16 min-h-screen bg-gradient-to-br from-blue-50 to-emerald-50 flex items-center justify-center">
          <div className="bg-white rounded-2xl shadow-xl p-8 max-w-md w-full mx-4">
            <div className="text-center">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
              <h2 className="text-2xl font-bold text-gray-900 mb-4">Loading Verification</h2>
              <p className="text-gray-600">Please wait while we load the verification page...</p>
            </div>
          </div>
        </div>
      }
    >
      <VerifyPageContent />
    </Suspense>
  );
};

export default VerifyPage;
