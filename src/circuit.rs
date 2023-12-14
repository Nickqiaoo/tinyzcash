use crate::coin::Coin;
use halo2_gadgets::sha256::{BlockWord,Bits,AssignedBits, Table16Chip, Table16Config};
use halo2_proofs::{
    arithmetic::Field,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::CurveAffine, pallas, vesta};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InstanceX {
    pub rt: Vec<u8>,
    pub old_sn: Vec<u8>,
    pub new_cm: Vec<u8>,
    pub public_value: u64,
    pub h_sig: Vec<u8>,
    pub h: Vec<u8>,
}

pub struct WitnessA {
    pub path: Vec<Vec<u8>>,
    pub old_coin: Coin,
    pub secret_key: String,
    pub new_coin: Coin,
}

pub fn create_proof(x: &InstanceX, a: &WitnessA) -> Vec<u8> {
    vec![]
}

#[derive(Debug, Clone)]
struct CircuitConfig {
    advice: Column<Advice>,
    instance: Column<Instance>,
    sha_config: Table16Config,
}

#[derive(Default)]
struct PourCircuit {
    pk: [BlockWord;16],
    sk: [BlockWord;8],
}

impl PourCircuit {
    fn load_private(
        &self, 
        config: &CircuitConfig,
        mut layouter: impl Layouter<pallas::Base>, 
        values: [BlockWord;16]
    ) -> Result<Vec<AssignedBits<32>>, Error> {
        layouter.assign_region(
            || "assign private values", 
            |mut region| {
                values
                    .iter()
                    .enumerate()
                    .map(|(i, value)| {
                        // Check that each cell of the input is a binary value
                        // Assign the private input value to an advice cell
                        region
                            .assign_advice(|| "assign private input", config.advice, i, || -> Bits<32> {value.0.into()})
                        }
                    )
                    .collect()
            }
        )
    }
}


impl Circuit<pallas::Base> for PourCircuit {
    type Config = CircuitConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
        let sha_config = Table16Chip::configure(meta);

        let advice = meta.advice_column();
        let instance = meta.instance_column();

        meta.enable_equality(instance);
        meta.enable_equality(advice);

        CircuitConfig {
            advice,
            instance,
            sha_config,
        }
    }
    
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), Error> {
        Table16Chip::load(config.sha_config.clone(), &mut layouter)?;
        let table16_chip = Table16Chip::construct(config.sha_config);

        let a = self.load_private(&config, layouter, self.pk);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::{dev::MockProver, pasta::Fp};
    #[test]
    fn test_chap_1() {
        // ANCHOR: test-circuit
        // The number of rows in our circuit cannot exceed 2^k. Since our example
        // circuit is very small, we can pick a very small value here.
        let k = 5;

        // Prepare the private and public inputs to the circuit!
        let c = Fp::from(1);
        let a = Fp::from(2);
        let b = Fp::from(3);
        let out = c * a.square() * b.square();
        println!("out=:{:?}", out);

        // Instantiate the circuit with the private inputs.
        let circuit = PourCircuit {};

        // Arrange the public input. We expose the multiplication result in row 0
        // of the instance column, so we position it there in our public inputs.
        let mut public_inputs = vec![out];

        // Given the correct public input, our circuit will verify.
        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));

        // If we try some other public input, the proof will fail!
        public_inputs[0] += Fp::one();
        let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
        assert!(prover.verify().is_err());
        println!("\n\n\n!!!!!OHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH!!!!!\n     simple example success !\n!!!!!OHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH!!!!!\n\n\n")
        // ANCHOR_END: test-circuit
    }

    #[cfg(feature = "dev-graph")]
    //cargo test plot_chap_1_circuit --features dev-graph
    #[test]
    fn plot_chap_1_circuit() {
        // Instantiate the circuit with the private inputs.
        let circuit = PourCircuit::<Fp>::default();
        // Create the area you want to draw on.
        // Use SVGBackend if you want to render to .svg instead.
        use plotters::prelude::*;
        let root = BitMapBackend::new("./chap_1_simple.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Simple Circuit without chip", ("sans-serif", 60))
            .unwrap();
        halo2_proofs::dev::CircuitLayout::default()
            // You can optionally render only a section of the circuit.
            // .view_width(0..2)
            // .view_height(0..16)
            // You can hide labels, which can be useful with smaller areas.
            .show_labels(true)
            // Render the circuit onto your area!
            // The first argument is the size parameter for the circuit.
            .render(5, &circuit, &root)
            .unwrap();
    }
}
