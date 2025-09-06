import { ethers } from "ethers";

export const getEvents = (contract: ethers.Contract, receipt: ethers.TransactionReceipt, eventName: string) => {
  try {
    const logs = receipt.logs;
    const eventInterface = contract.interface;

    for (const log of logs) {
      try {
        const parsedLog = eventInterface.parseLog({
          topics: log.topics,
          data: log.data,
        });

        if (parsedLog && parsedLog.name === eventName) {
          // Return the event args as an object
          const eventArgs: { [key: string]: any } = {};

          // Convert the args array to an object with named properties
          if (parsedLog.args) {
            for (let i = 0; i < parsedLog.args.length; i++) {
              const fragment = parsedLog.fragment.inputs[i];
              if (fragment) {
                eventArgs[fragment.name] = parsedLog.args[i];
              }
            }
          }

          return eventArgs;
        }
      } catch (error) {
        console.error(error);
        // Skip logs that can't be parsed by this contract
        continue;
      }
    }

    throw new Error(`Event ${eventName} not found in transaction receipt`);
  } catch (error) {
    console.error("Error parsing events:", error);
    throw error;
  }
};
