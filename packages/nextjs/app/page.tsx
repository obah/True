"use client";

// const Home: NextPage = () => {
//   const { address: connectedAddress } = useAccount();
//   return (
//     <>
//       <div className="flex items-center flex-col grow pt-10">
//         <div className="px-5">
//           <h1 className="text-center">
//             <span className="block text-2xl mb-2">Welcome to</span>
//             <span className="block text-4xl font-bold">Scaffold-ETH 2</span>
//           </h1>
//           <div className="flex justify-center items-center space-x-2 flex-col">
//             <p className="my-2 font-medium">Connected Address:</p>
//             <Address address={connectedAddress} />
//           </div>
//           <p className="text-center text-lg">
//             Get started by editing{" "}
//             <code className="italic bg-base-300 text-base font-bold max-w-full break-words break-all inline-block">
//               packages/nextjs/app/page.tsx
//             </code>
//           </p>
//           <p className="text-center text-lg">
//             Edit your smart contract{" "}
//             <code className="italic bg-base-300 text-base font-bold max-w-full break-words break-all inline-block">
//               YourContract.sol
//             </code>{" "}
//             in{" "}
//             <code className="italic bg-base-300 text-base font-bold max-w-full break-words break-all inline-block">
//               packages/hardhat/contracts
//             </code>
//           </p>
//         </div>
//         <div className="grow bg-base-300 w-full mt-16 px-8 py-12">
//           <div className="flex justify-center items-center gap-12 flex-col md:flex-row">
//             <div className="flex flex-col bg-base-100 px-10 py-10 text-center items-center max-w-xs rounded-3xl">
//               <BugAntIcon className="h-8 w-8 fill-secondary" />
//               <p>
//                 Tinker with your smart contract using the{" "}
//                 <Link href="/debug" passHref className="link">
//                   Debug Contracts
//                 </Link>{" "}
//                 tab.
//               </p>
//             </div>
//             <div className="flex flex-col bg-base-100 px-10 py-10 text-center items-center max-w-xs rounded-3xl">
//               <MagnifyingGlassIcon className="h-8 w-8 fill-secondary" />
//               <p>
//                 Explore your local transactions with the{" "}
//                 <Link href="/blockexplorer" passHref className="link">
//                   Block Explorer
//                 </Link>{" "}
//                 tab.
//               </p>
//             </div>
//           </div>
//         </div>
//       </div>
//     </>
//   );
// };
// export default Home;
import React from "react";
import Link from "next/link";
import { ArrowRight, Factory, Lock, Scan, Shield, Users, Zap } from "lucide-react";

const LandingPage = () => {
  const features = [
    {
      icon: Shield,
      title: "Blockchain Security",
      description: "Immutable product certificates secured by blockchain technology",
      color: "from-blue-500 to-blue-600",
    },
    {
      icon: Scan,
      title: "QR Code Verification",
      description: "Instantly verify product authenticity with QR code scanning",
      color: "from-emerald-500 to-emerald-600",
    },
    {
      icon: Lock,
      title: "Ownership Transfer",
      description: "Secure and transparent ownership transfer between users",
      color: "from-orange-500 to-orange-600",
    },
    {
      icon: Zap,
      title: "Real-time Tracking",
      description: "Track product lifecycle and ownership history in real-time",
      color: "from-purple-500 to-purple-600",
    },
  ];

  const stats = [
    { number: "10K+", label: "Products Verified" },
    { number: "500+", label: "Manufacturers" },
    { number: "50K+", label: "Users Protected" },
    { number: "99.9%", label: "Accuracy Rate" },
  ];

  return (
    <div className="pt-16">
      {/* Hero Section */}
      <section className="relative overflow-hidden bg-gradient-to-br from-blue-50 via-white to-emerald-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900 transition-colors duration-300">
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg width=%2260%22 height=%2260%22 viewBox=%220 0 60 60%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cg fill=%22none%22 fill-rule=%22evenodd%22%3E%3Cg fill=%22%239C92AC%22 fill-opacity=%220.05%22%3E%3Ccircle cx=%2230%22 cy=%2230%22 r=%221%22/%3E%3C/g%3E%3C/g%3E%3C/svg%3E')] opacity-40"></div>

        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 lg:py-28">
          <div className="text-center">
            <div className="inline-flex items-center space-x-2 bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300 px-4 py-2 rounded-full text-sm font-medium mb-8 transition-colors duration-300">
              <Shield className="h-4 w-4" />
              <span>Powered by Blockchain Technology</span>
            </div>

            <h1 className="text-4xl md:text-6xl lg:text-7xl font-bold text-gray-900 dark:text-white mb-6 leading-tight transition-colors duration-300">
              <span className="bg-gradient-to-r from-blue-600 via-purple-600 to-emerald-600 bg-clip-text text-transparent">
                Verify. Protect. Own.
              </span>
            </h1>

            <p className="text-xl md:text-2xl text-gray-600 dark:text-slate-300 mb-10 max-w-4xl mx-auto leading-relaxed transition-colors duration-300">
              Revolutionary blockchain-based platform that ensures product authenticity and secure ownership transfer.
              Protect your brand and customers with immutable verification technology.
            </p>

            <div className="flex flex-col sm:flex-row items-center justify-center space-y-4 sm:space-y-0 sm:space-x-6">
              <Link
                href="/manufacturer"
                className="group inline-flex items-center space-x-3 bg-gradient-to-r from-blue-600 to-blue-700 text-white px-8 py-4 rounded-xl font-semibold text-lg hover:from-blue-700 hover:to-blue-800 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
              >
                <Factory className="h-5 w-5" />
                <span>For Manufacturers</span>
                <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform duration-300" />
              </Link>

              <Link
                href="/users"
                className="group inline-flex items-center space-x-3 bg-white dark:bg-slate-800 text-gray-700 dark:text-slate-200 px-8 py-4 rounded-xl font-semibold text-lg border-2 border-gray-200 dark:border-slate-600 hover:border-emerald-300 dark:hover:border-emerald-500 hover:text-emerald-700 dark:hover:text-emerald-400 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
              >
                <Users className="h-5 w-5" />
                <span>For Users</span>
                <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform duration-300" />
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-16 bg-white dark:bg-slate-800 border-y border-gray-100 dark:border-slate-700 transition-colors duration-300">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-8">
            {stats.map((stat, index) => (
              <div key={index} className="text-center">
                <div className="text-3xl lg:text-4xl font-bold text-gray-900 dark:text-white mb-2 transition-colors duration-300">
                  {stat.number}
                </div>
                <div className="text-gray-600 dark:text-slate-300 font-medium transition-colors duration-300">
                  {stat.label}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-gradient-to-br from-gray-50 to-blue-50 dark:from-slate-900 dark:to-slate-800 transition-colors duration-300">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-gray-900 dark:text-white mb-6 transition-colors duration-300">
              Why Choose True?
            </h2>
            <p className="text-xl text-gray-600 dark:text-slate-300 max-w-3xl mx-auto transition-colors duration-300">
              Engineered with advanced blockchain technology, delivering unparalleled security, transparency, and trust
              for verifying product authenticity and establishing provable ownership.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            {features.map((feature, index) => (
              <div
                key={index}
                className="group bg-white dark:bg-slate-800 rounded-2xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-2"
              >
                <div
                  className={`inline-flex p-4 rounded-xl bg-gradient-to-r ${feature.color} mb-6 group-hover:scale-110 transition-transform duration-300`}
                >
                  <feature.icon className="h-8 w-8 text-white" />
                </div>
                <h3 className="text-xl font-bold text-gray-900 dark:text-white mb-4 transition-colors duration-300">
                  {feature.title}
                </h3>
                <p className="text-gray-600 dark:text-slate-300 leading-relaxed transition-colors duration-300">
                  {feature.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* How It Works Section */}
      <section className="py-20 bg-white dark:bg-slate-900 transition-colors duration-300">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-gray-900 dark:text-white mb-6 transition-colors duration-300">
              How It Works
            </h2>
            <p className="text-xl text-gray-600 dark:text-slate-300 max-w-3xl mx-auto transition-colors duration-300">
              Simple, secure, and transparent process for both manufacturers and users.
            </p>
          </div>

          <div className="grid lg:grid-cols-2 gap-16 items-center">
            {/* Manufacturer Flow */}
            <div>
              <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-8 flex items-center transition-colors duration-300">
                <Factory className="h-6 w-6 mr-3 text-emerald-600" />
                For Manufacturers
              </h3>
              <div className="space-y-6">
                {[
                  "Register as a verified manufacturer on the platform",
                  "Create and sign digital certificates for your products",
                  "Generate QR codes containing authenticity data",
                  "Customers can verify products instantly",
                ].map((step, index) => (
                  <div key={index} className="flex items-start space-x-4">
                    <div className="flex-shrink-0 w-8 h-8 bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600 dark:text-emerald-400 rounded-full flex items-center justify-center font-bold text-sm transition-colors duration-300">
                      {index + 1}
                    </div>
                    <p className="text-gray-700 dark:text-slate-300 leading-relaxed transition-colors duration-300">
                      {step}
                    </p>
                  </div>
                ))}
              </div>
            </div>

            {/* User Flow */}
            <div>
              <h3 className="text-2xl font-bold text-gray-900 dark:text-white mb-8 flex items-center transition-colors duration-300">
                <Users className="h-6 w-6 mr-3 text-blue-600" />
                For Users
              </h3>
              <div className="space-y-6">
                {[
                  "Scan QR code on product packaging",
                  "Instantly verify product authenticity",
                  "Claim ownership of verified products",
                  "Transfer ownership securely to others",
                ].map((step, index) => (
                  <div key={index} className="flex items-start space-x-4">
                    <div className="flex-shrink-0 w-8 h-8 bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 rounded-full flex items-center justify-center font-bold text-sm transition-colors duration-300">
                      {index + 1}
                    </div>
                    <p className="text-gray-700 dark:text-slate-300 leading-relaxed transition-colors duration-300">
                      {step}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-gradient-to-r from-blue-600 via-purple-600 to-emerald-600">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-white mb-6 transition-colors duration-300">
            Ready to Secure Your Products?
          </h2>
          <p className="text-xl text-blue-100 dark:text-blue-200 mb-10 max-w-3xl mx-auto transition-colors duration-300">
            Join thousands of manufacturers and users who trust True for product verification and ownership management.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center space-y-4 sm:space-y-0 sm:space-x-6">
            <Link
              href="/manufacturer"
              className="group inline-flex items-center space-x-3 bg-white text-blue-600 px-8 py-4 rounded-xl font-semibold text-lg hover:bg-gray-50 transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
            >
              <Factory className="h-5 w-5" />
              <span>Start as Manufacturer</span>
              <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform duration-300" />
            </Link>

            <Link
              href="/users"
              className="group inline-flex items-center space-x-3 bg-transparent text-white px-8 py-4 rounded-xl font-semibold text-lg border-2 border-white hover:bg-white hover:text-blue-600 dark:hover:bg-slate-800 dark:hover:text-white transition-all duration-300 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
            >
              <Users className="h-5 w-5" />
              <span>Start as User</span>
              <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform duration-300" />
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="bg-gray-900 dark:bg-slate-950 text-white py-12 transition-colors duration-300">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col md:flex-row items-center justify-between">
            <div className="flex items-center space-x-3 mb-4 md:mb-0">
              <div className="p-2 bg-gradient-to-br from-blue-600 to-blue-700 rounded-xl">
                <Shield className="h-6 w-6 text-white" />
              </div>
              <span className="text-xl font-bold">True</span>
            </div>
            <p className="text-gray-400 dark:text-slate-400 text-center md:text-right transition-colors duration-300">
              © 2025 True. Securing authenticity with blockchain technology.
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default LandingPage;
