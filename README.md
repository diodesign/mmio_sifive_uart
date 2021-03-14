## mmio_16550_uart

This is a very basic Rust `no_std` crate for reading from and writing to a memory-mapped (MMIO) SiFive Universal Asynchronous Receiver/Transmitter (UART) at the bare-metal level. It is used by the [Diosix](https://diosix.org) project for serial port communication. It is compatible with these [SiFive](https://www.sifive.com/boards) system-on-chips:

* FU740-C000 (used in the HiFive Unmatched)
* FU540-C000 (used in the HiFive Unleashed)
* FE310-G002 (used in the HiFive1)

### Contact and code of conduct <a name="contact"></a>

Please [email](mailto:chrisw@diosix.org) project lead Chris Williams if you have any questions or issues to raise, wish to get involved, have source to contribute, or have found a security flaw. You can, of course, submit pull requests or raise issues via GitHub, though please consider disclosing security-related matters privately. Please also observe the Diosix project's [code of conduct](https://diosix.org/docs/conduct.html) if you wish to participate.

### Copyright and license <a name="copyright"></a>

Copyright &copy; Chris Williams, 2021. See [LICENSE](LICENSE) for distribution and use of source code and binaries.
