const { planCheckInputs } = await import(process.argv[2]);
console.log(JSON.stringify(planCheckInputs([process.argv[3]])));
