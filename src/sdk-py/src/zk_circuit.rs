use crate::base::Base;
use crate::point::Point;
use crate::scalar::Scalar;
use crate::zk_binary::ZkBinary;
use darkfi::zk::{halo2::Value, vm, vm_heap::empty_witnesses};
use darkfi_sdk::crypto::MerkleNode;
use pyo3::prelude::*;
use std::ops::Deref;

#[pyclass]
pub struct ZkCircuit(pub(crate) vm::ZkCircuit, pub(crate) Vec<vm::Witness>);

/// QUESTION: how to deal with witness?
/// Like Builder Object
#[pymethods]
impl ZkCircuit {
    #[new]
    fn new(circuit_code: &PyCell<ZkBinary>) -> Self {
        let circuit_code = circuit_code.borrow().deref().0.clone();
        // DUMMY CIRCUIT
        let circuit = vm::ZkCircuit::new(vec![], circuit_code.clone());
        Self(circuit, vec![])
    }

    fn build(&self, circuit_code: &PyCell<ZkBinary>) -> Self {
        let circuit_code = circuit_code.borrow().deref().0.clone();
        let circuit = vm::ZkCircuit::new(self.1.clone(), circuit_code.clone());
        Self(circuit, self.1.clone())
    }

    fn verifier_build(&self, circuit_code: &PyCell<ZkBinary>) -> Self {
        let circuit_code = circuit_code.borrow().deref().0.clone();
        let circuit = vm::ZkCircuit::new(empty_witnesses(&circuit_code), circuit_code.clone());
        Self(circuit, self.1.clone())
    }

    fn witness_point(&mut self, v: &PyCell<Point>) {
        let v = v.borrow();
        let v = v.deref();
        self.1.push(vm::Witness::EcPoint(Value::known(v.0)));
    }

    fn witness_ni_point(&mut self, v: &PyCell<Point>) {
        let v = v.borrow();
        let v = v.deref();
        self.1.push(vm::Witness::EcNiPoint(Value::known(v.0)));
    }

    fn witness_fixed_point(&mut self, v: &PyCell<Point>) {
        let v = v.borrow();
        let v = v.deref();
        self.1.push(vm::Witness::EcFixedPoint(Value::known(v.0)));
    }

    fn witness_scalar(&mut self, v: &PyCell<Scalar>) {
        let v = v.borrow();
        let v = v.deref();
        self.1.push(vm::Witness::Scalar(Value::known(v.0)));
    }

    fn witness_base(&mut self, v: &PyCell<Base>) {
        let v = v.borrow();
        let v = v.deref();
        self.1.push(vm::Witness::Base(Value::known(v.0)));
    }

    fn witness_merkle_path(&mut self, v: Vec<&PyCell<Base>>) {
        let v: Vec<MerkleNode> = v.iter().map(|v| MerkleNode::new(v.borrow().deref().0)).collect();
        let v: [MerkleNode; 32] = v.try_into().unwrap();
        let v = Value::known(v);
        self.1.push(vm::Witness::MerklePath(v));
    }

    fn witness_u32(&mut self, v: u32) {
        self.1.push(vm::Witness::Uint32(Value::known(v)));
    }

    fn witness_u64(&mut self, v: u64) {
        self.1.push(vm::Witness::Uint64(Value::known(v)));
    }
}

pub fn create_module(py: pyo3::Python<'_>) -> pyo3::PyResult<&PyModule> {
    let submod = PyModule::new(py, "zk_circuit")?;
    submod.add_class::<ZkCircuit>()?;
    Ok(submod)
}
