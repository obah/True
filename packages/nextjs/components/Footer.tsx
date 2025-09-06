import React from "react";
import { Shield } from "lucide-react";

/**
 * Site footer
 */
export const Footer = () => {
  return (
    <footer className="bg-gray-900 dark:bg-slate-950 text-white py-4 transition-colors duration-300">
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
  );
};
