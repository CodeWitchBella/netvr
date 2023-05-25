using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class RemoteClientSnapshot: IEquatable<RemoteClientSnapshot>, ICloneable {
        public RemoteConfigurationSnapshot configuration;
        public StateSnapshot state;

        public RemoteClientSnapshot(RemoteConfigurationSnapshot _configuration, StateSnapshot _state) {
            if (_configuration == null) throw new ArgumentNullException(nameof(_configuration));
            configuration = _configuration;
            if (_state == null) throw new ArgumentNullException(nameof(_state));
            state = _state;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            configuration.Serialize(serializer);
            state.Serialize(serializer);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static RemoteClientSnapshot Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RemoteClientSnapshot obj = new RemoteClientSnapshot(
            	RemoteConfigurationSnapshot.Deserialize(deserializer),
            	StateSnapshot.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RemoteClientSnapshot BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RemoteClientSnapshot BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RemoteClientSnapshot value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RemoteClientSnapshot other && Equals(other);

        public static bool operator ==(RemoteClientSnapshot left, RemoteClientSnapshot right) => Equals(left, right);

        public static bool operator !=(RemoteClientSnapshot left, RemoteClientSnapshot right) => !Equals(left, right);

        public bool Equals(RemoteClientSnapshot other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!configuration.Equals(other.configuration)) return false;
            if (!state.Equals(other.state)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + configuration.GetHashCode();
                value = 31 * value + state.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RemoteClientSnapshot Clone() => (RemoteClientSnapshot)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
